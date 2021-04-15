use futures::stream::TryStreamExt;
use ipnetwork::IpNetwork;
use rtnetlink::{new_connection, sys, Error, NetworkNamespace};

pub fn veth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let links_veth = vec!["po-veth-1", "po-veth-2"];

    //Create pair veth
    let create_veth = async {
        if let Err(e) = create_veth(links_veth[0].to_string(), links_veth[1].to_string()).await {
            eprintln!("{}", e);
        };
    };
    rt.block_on(create_veth);

    for (i, link) in links_veth.iter().enumerate() {
        let ip = format!("10.1.11.{}/24", i);
        let ip_veth: IpNetwork = ip.parse().unwrap_or_else(|_| {
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

pub fn bridge() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let links_bridge = vec!["po-bridge-1"];
    let links_veth = vec!["po-veth-1", "po-veth-2"];

    //Create
    let create_bridge = async {
        if let Err(e) = create_bridge(links_bridge[0].to_string()).await {
            eprintln!("{}", e);
        };
    };
    rt.block_on(create_bridge);

    for (_, link) in links_bridge.iter().enumerate() {
        let ip = format!("10.1.11.3/24");
        let ip_veth: IpNetwork = ip.parse().unwrap_or_else(|_| {
            eprint!("invalid address");
            std::process::exit(1);
        });

        //Add Ip address to each veth
        let add_bridge_address = async {
            if let Err(e) = add_link_address(link.to_string(), ip_veth).await {
                eprintln!("{}", e);
            }
        };
        rt.block_on(add_bridge_address);

        //Make each veth up
        let set_bridge_up = async {
            if let Err(e) = set_link_up(link.to_string()).await {
                eprintln!("{}", e);
            }
        };
        rt.block_on(set_bridge_up);
    }

    // let set_veth_to_bridge = async {
    //     if let Err(e) =
    //         set_veth_to_bridge(links_veth[0].to_string(), links_bridge[0].to_string()).await
    //     {
    //         eprintln!("{}", e);
    //     }
    // };
    // rt.block_on(set_veth_to_bridge);
}

pub fn netns() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let netns = vec!["neteto1"];

    for (_, ns) in netns.iter().enumerate() {
        let veth_up = async {
            if let Err(e) = create_netns(ns.to_string()).await {
                eprintln!("{}", e);
            }
        };
        rt.block_on(veth_up);
    }
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

//Not work yet
async fn set_veth_to_bridge(link_veth: String, link_bridge: String) -> Result<(), Error> {
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);
    let mut links_veth = handle.link().get().set_name_filter(link_veth).execute();
    let mut links_bridge = handle.link().get().set_name_filter(link_bridge).execute();
    if let Some(link1) = links_veth.try_next().await? {
        if let Some(link2) = links_bridge.try_next().await? {
            handle
                .link()
                .set(link1.header.index)
                .master(link2.header.index);
        }
    }
    Ok(())
}

async fn create_netns(ns_name: String) -> Result<(), String> {
    NetworkNamespace::add(ns_name.to_string())
        .await
        .map_err(|e| format!("{}", e))
}

// pub async fn dump_addresses() -> Result<(), Error> {
//     let (connection, handle, _) = new_connection().unwrap();
//     tokio::spawn(connection);

//     let link = "veth-po-1".to_string();
//     println!("dumping address for link \"{}\"", link);

//     let mut links = handle.link().get().set_name_filter(link.clone()).execute();
//     if let Some(link) = links.try_next().await? {
//         let mut addresses = handle
//             .address()
//             .get()
//             .set_link_index_filter(link.header.index)
//             .execute();
//         while let Some(msg) = addresses.try_next().await? {
//             println!("{:?}", msg);
//         }
//         Ok(())
//     } else {
//         eprintln!("link {} not found", link);
//         Ok(())
//     }
// }
