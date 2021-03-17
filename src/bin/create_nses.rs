use potato::namespace::{self, create_nses, Namespace};

fn main() {
    let nses = vec![Namespace::Uts, Namespace::Network];
    for ns in &nses {
        let fd = namespace::get_proc_ns_fd(*ns).unwrap();
        println!("fd of namespace {} is {}", ns.to_proc_ns_name(), fd);
    }

    let result = create_nses(nses).unwrap();
    for (ns, fd) in &result {
        println!("fd of namespace {} is {}", ns.to_proc_ns_name(), fd);
    }
}
