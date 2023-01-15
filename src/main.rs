use utils::timer::Timer;

mod day01_calorie_counting;
mod day02_rock_paper_scissors;
mod day03_rucksack_reorganization;
mod day04_camp_cleanup;
mod day05_supply_stacks;
mod day06_tuning_trouble;

fn main() {
    env_logger::init();
    let _timer = Timer::start(|elapsed| println!("main took {} ms.", elapsed.as_millis()));
    let day: usize = if let Some(arg1) = std::env::args().nth(1) {
        arg1.parse().expect("argument should be an integer")
    } else {
        6
    };
    println!("running day {}\n", day);
    match day {
        1 => day01_calorie_counting::run(),
        2 => day02_rock_paper_scissors::run(),
        3 => day03_rucksack_reorganization::run(),
        4 => day04_camp_cleanup::run(),
        5 => day05_supply_stacks::run(),
        6 => day06_tuning_trouble::run(),
        _ => panic!("day {} not found", day),
    }
}
