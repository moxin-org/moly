fn main() {
    println!("Moxin was invoked with args: {:#?}", std::env::args().collect::<Vec<_>>());

    moxin::app::app_main()
}
