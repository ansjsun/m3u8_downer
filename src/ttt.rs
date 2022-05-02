trait Foo {
    fn foo(value: u8) -> u8;
}

trait Bar {
    const BAR: u8;

    fn bar(value: &str) -> u8;
}

const fn baz<T>() -> u8
where
    T: Foo + ~const Bar,
{
    T::foo(T::BAR) // error: `<T as Bar>` used in a const context
}

fn main() {
    println!("hello");
}
