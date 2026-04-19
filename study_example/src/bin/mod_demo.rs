//访问父级目录的库得通过crate来访问
use study_example::mod01::hosting;

fn serve_order() {
    self::back_of_house::cook_order();
    back_of_house::cook_order();
}

// 厨房模块
mod back_of_house {
    fn fix_incorrect_order() {
        cook_order();
        super::serve_order();
    }
    pub fn cook_order() {}
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        println!("Hello, world!");
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

fn main() {
    println!("Hello, world!");
    study_example::mod01::hosting::add_to_waitlist();
}
