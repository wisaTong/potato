use futures::stream::TryStreamExt;
use ipnetwork::IpNetwork;
use rtnetlink::{new_connection, Error, NetworkNamespace};

const VETHS_NAME: [&'static str; 2] = ["poeth1", "poeth2"];
const BRIDGE_NAME: &'static str = "bridge";

// Create veth pair one for set master to bridge, other for setns to clone process.
// poeth1 always set master to bridge
// poeth2 always move to clone process
pub fn prep_network_stack() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    //Create pair veth
    let create_veth = async {
        if let Err(e) = create_veth(VETHS_NAME[0].to_string(), VETHS_NAME[1].to_string()).await {
            eprintln!("{}", e);
        };
    };
    rt.block_on(create_veth);

    //Create bridge
    let create_bridge = async {
        if let Err(e) = create_bridge(BRIDGE_NAME.to_string()).await {
            eprintln!("{}", e);
        };
    };
    rt.block_on(create_bridge);

    let set_veth_to_bridge = async {
        if let Err(e) = set_veth_to_bridge(VETHS_NAME[0].to_string(), BRIDGE_NAME.to_string()).await
        {
            eprintln!("{}", e);
        }
    };
    rt.block_on(set_veth_to_bridge);
}

// set network in parent process
// ***vip = valid ip

pub fn set_outside_network(ip: String, pid: u32) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let vip: IpNetwork = ip.parse().unwrap_or_else(|_| {
        eprint!("invalid address");
        std::process::exit(1);
    });
    let link_up_veth = async {
        if let Err(e) = set_link_up(VETHS_NAME[0].to_string()).await {
            eprintln!("{}", e);
        }
    };
    rt.block_on(link_up_veth);
    let link_up_br = async {
        if let Err(e) = set_link_up(BRIDGE_NAME.to_string()).await {
            eprintln!("{}", e);
        }
    };
    rt.block_on(link_up_br);
    let set_veth_to_bridge = async {
        if let Err(e) = set_veth_to_bridge(VETHS_NAME[0].to_string(), BRIDGE_NAME.to_string()).await
        {
            eprintln!("{}", e);
        }
    };
    rt.block_on(set_veth_to_bridge);
    let add_link_address = async {
        if let Err(e) = add_link_address(BRIDGE_NAME.to_string(), vip).await {
            eprintln!("{}", e);
        }
    };
    rt.block_on(add_link_address);
    let setns_by_pid = async {
        if let Err(e) = setns_by_pid(VETHS_NAME[1].to_string(), pid).await {
            eprintln!("{}", e);
        }
    };
    rt.block_on(setns_by_pid);
}

//set network in clone process
pub fn set_inside_network(ip: String) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let vip: IpNetwork = ip.parse().unwrap_or_else(|_| {
        eprint!("invalid address");
        std::process::exit(1);
    });
    let link_up_veth = async {
        if let Err(e) = set_link_up(VETHS_NAME[1].to_string()).await {
            eprintln!("{}", e);
        };
    };
    rt.block_on(link_up_veth);
    let add_link_address = async {
        if let Err(e) = add_link_address(VETHS_NAME[1].to_string(), vip).await {
            eprintln!("{}", e);
        }
    };
    rt.block_on(add_link_address);
}

//stand alone veth pair with ip address
pub fn prep_veth(ip: [&'static str; 2]) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    //Create pair veth
    let create_veth = async {
        if let Err(e) = create_veth(VETHS_NAME[0].to_string(), VETHS_NAME[1].to_string()).await {
            eprintln!("{}", e);
        };
    };
    rt.block_on(create_veth);

    for (i, link) in VETHS_NAME.iter().enumerate() {
        let ip_veth: IpNetwork = ip[i].parse().unwrap_or_else(|_| {
            eprint!("invalid address");
            std::process::exit(1);
        });

        //Add Ip address to each veth
        let add_veth_address = async {
            if let Err(e) = add_link_address(link.to_string(), ip_veth).await {
                eprintln!("{}", e);
            }
        };
        rt.block_on(add_veth_address);

        //Make each veth up
        let set_veth_up = async {
            if let Err(e) = set_link_up(link.to_string()).await {
                eprintln!("{}", e);
            }
        };
        rt.block_on(set_veth_up);
    }
}

//stand alone bridge with ip address
pub fn prep_bridge(ip: String) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    //Create
    let create_bridge = async {
        if let Err(e) = create_bridge(BRIDGE_NAME.to_string()).await {
            eprintln!("{}", e);
        };
    };
    rt.block_on(create_bridge);

    //Check Ip is valid
    let ip_veth: IpNetwork = ip.parse().unwrap_or_else(|_| {
        eprint!("invalid address");
        std::process::exit(1);
    });

    //Add Ip address to each veth
    let add_bridge_address = async {
        if let Err(e) = add_link_address(BRIDGE_NAME.to_string(), ip_veth).await {
            eprintln!("{}", e);
        }
    };
    rt.block_on(add_bridge_address);

    //Make each veth up
    let set_bridge_up = async {
        if let Err(e) = set_link_up(BRIDGE_NAME.to_string()).await {
            eprintln!("{}", e);
        }
    };
    rt.block_on(set_bridge_up);
}

async fn set_link_up(link: String) -> Result<(), Error> {
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);
    let mut links = handle.link().get().set_name_filter(link.clone()).execute();
    if let Some(link) = links.try_next().await? {
        handle.link().set(link.header.index).up().execute().await?
    } else {
        println!("no link link {} found", link);
    }
    Ok(())
}

async fn add_link_address(link: String, ip: IpNetwork) -> Result<(), Error> {
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);

    let mut links = handle.link().get().set_name_filter(link).execute();
    if let Some(link) = links.try_next().await? {
        handle
            .address()
            .add(link.header.index, ip.ip(), ip.prefix())
            .execute()
            .await?
    }
    Ok(())
}

async fn create_veth(links_veth1: String, links_veth2: String) -> Result<(), String> {
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);
    handle
        .link()
        .add()
        .veth(links_veth1.into(), links_veth2.into())
        .execute()
        .await
        .map_err(|e| format!("{}", e))
}

async fn create_bridge(links_bridge: String) -> Result<(), String> {
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);
    handle
        .link()
        .add()
        .bridge(links_bridge.into())
        .execute()
        .await
        .map_err(|e| format!("{}", e))
}

async fn set_veth_to_bridge(link_veth: String, link_bridge: String) -> Result<(), Error> {
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);
    let mut links_veth = handle.link().get().set_name_filter(link_veth).execute();
    if let Some(link1) = links_veth.try_next().await? {
        let mut links_bridge = handle.link().get().set_name_filter(link_bridge).execute();
        if let Some(link2) = links_bridge.try_next().await? {
            handle
                .link()
                .set(link1.header.index)
                .master(link2.header.index)
                .execute()
                .await?;
        }
    }
    Ok(())
}

async fn setns_by_pid(device_name: String, pid: u32) -> Result<(), rtnetlink::Error> {
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);
    let mut links_device = handle.link().get().set_name_filter(device_name).execute();
    if let Some(link) = links_device.try_next().await? {
        handle
            .link()
            .set(link.header.index)
            .setns_by_pid(pid)
            .execute()
            .await?;
    }
    Ok(())
}

async fn create_netns(ns_name: String) -> Result<(), String> {
    NetworkNamespace::add(ns_name)
        .await
        .map_err(|e| format!("{}", e))
}