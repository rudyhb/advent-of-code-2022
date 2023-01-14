use utils::timer::Timer;

mod day01_calorie_counting;
mod day02_rock_paper_scissors;
mod day03_rucksack_reorganization;
mod day04_camp_cleanup;

fn main() {
    env_logger::init();
    let _timer = Timer::start(|elapsed| println!("main took {} ms.", elapsed.as_millis()));
    let day: usize = if let Some(arg1) = std::env::args().nth(1) {
        arg1.parse().expect("argument should be an integer")
    } else {
        4
    };
    println!("running day {}\n", day);
    match day {
        1 => day01_calorie_counting::run(),
        2 => day02_rock_paper_scissors::run(),
        3 => day03_rucksack_reorganization::run(),
        4 => day04_camp_cleanup::run(),
        _ => panic!("day {} not found", day),
    }
}
