fn main() {
    struct NontrivialDrop;
    impl Drop for NontrivialDrop {
        fn drop(&mut self) {
            println!("Dropped!");
        }
    }

    let nontrivial_drop = NontrivialDrop;
    let x = 5;
    let f = Func::new(move |y| {
        let _ = &nontrivial_drop;
        println!("{}", x + y);
    });

    f(7);
}
