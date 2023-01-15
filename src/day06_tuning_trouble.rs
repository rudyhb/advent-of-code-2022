use std::collections::HashSet;

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input6.txt").unwrap();
    println!(
        "characters before start-of-packet mark is detected: {}",
        get_start_of_packet_position(&input, 4)
    );
    println!(
        "characters before start-of-message mark is detected: {}",
        get_start_of_packet_position(&input, 14)
    );
}

fn get_start_of_packet_position(s: &str, marker_size: usize) -> usize {
    let chars = s.chars().enumerate().collect::<Vec<_>>();
    chars
        .windows(marker_size)
        .filter(|chars| chars.iter().map(|c| c.1).collect::<HashSet<char>>().len() == marker_size)
        .next()
        .expect("start of packet position not found")[marker_size - 1]
        .0
        + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        assert_eq!(
            get_start_of_packet_position("bvwbjplbgvbhsrlpgdmjqwftvncz", 4),
            5
        );
        assert_eq!(
            get_start_of_packet_position("nppdvjthqldpwncqszvftbrmjlhg", 4),
            6
        );
        assert_eq!(
            get_start_of_packet_position("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg", 4),
            10
        );
        assert_eq!(
            get_start_of_packet_position("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw", 4),
            11
        );
    }

    #[test]
    fn test2() {
        assert_eq!(
            get_start_of_packet_position("mjqjpqmgbljsphdztnvjfqwrcgsmlb", 14),
            19
        );
        assert_eq!(
            get_start_of_packet_position("bvwbjplbgvbhsrlpgdmjqwftvncz", 14),
            23
        );
        assert_eq!(
            get_start_of_packet_position("nppdvjthqldpwncqszvftbrmjlhg", 14),
            23
        );
        assert_eq!(
            get_start_of_packet_position("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg", 14),
            29
        );
        assert_eq!(
            get_start_of_packet_position("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw", 14),
            26
        );
    }
}
