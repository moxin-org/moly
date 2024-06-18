fn main() {
    println!("------------------------- Environment Variables -------------------------");
    println!("{:#?}", std::env::vars().collect::<Vec<_>>());

    moxin::app::app_main()
}
