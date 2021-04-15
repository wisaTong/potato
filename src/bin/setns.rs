use potato::net;
fn main() {
    net::veth();
    net::bridge();
    net::setns(5572);
}
