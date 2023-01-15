use std::collections::HashMap;

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input7.txt").unwrap();

    let dir = build_directories(&input);
    println!(
        "sum of total sizes of small directories: {}",
        dir.get_directories_recursive()
            .into_iter()
            .filter(|d| d.cached_size <= 100_000)
            .map(|d| d.cached_size)
            .sum::<u64>()
    );

    let used = dir.cached_size;
    println!(
        "initial specs: total {}, used {}, free {}",
        Directory::TOTAL_SPACE,
        used,
        Directory::TOTAL_SPACE - used,
    );

    println!(
        "smallest directory that can be deleted: {}",
        dir.get_smallest_directory_big_enough().cached_size
    );
}

fn build_directories(s: &str) -> Directory {
    let instructions = parse_input(s);

    let mut dir = Directory::new("/".to_string());
    let mut parents: Vec<Directory> = Vec::new();
    fn return_to_root(dir: &mut Directory, parents: &mut Vec<Directory>) {
        while try_change_to_parent_dir(dir, parents) {}
    }
    fn try_change_to_parent_dir(dir: &mut Directory, parents: &mut Vec<Directory>) -> bool {
        if let Some(mut parent) = parents.pop() {
            std::mem::swap(&mut parent, dir);
            dir.directories.insert(parent.name.to_string(), parent);
            true
        } else {
            false
        }
    }
    for instruction in instructions {
        match instruction {
            Instruction::ChangeToRoot => {
                return_to_root(&mut dir, &mut parents);
            }
            Instruction::ChangeTo(name) => {
                let next = dir
                    .directories
                    .remove(&name)
                    .expect("cannot find directory to move to");
                parents.push(dir);
                dir = next;
            }
            Instruction::ChangeToPrevious => {
                try_change_to_parent_dir(&mut dir, &mut parents);
            }
            Instruction::List(list) => {
                for (size, name) in list {
                    if let Some(size) = size {
                        dir.files.insert(name.to_string(), size);
                    } else {
                        dir.directories
                            .insert(name.to_string(), Directory::new(name.to_string()));
                    }
                }
            }
        }
    }
    return_to_root(&mut dir, &mut parents);
    dir.update_cached_size();
    dir
}

fn parse_input(s: &str) -> Vec<Instruction> {
    let s = s.trim_start_matches("$ ");
    s.split("\n$ ")
        .map(|c| {
            let mut words = c.split_whitespace();
            match words.next().expect("invalid command") {
                "cd" => match words.next().unwrap() {
                    "/" => Instruction::ChangeToRoot,
                    ".." => Instruction::ChangeToPrevious,
                    name => Instruction::ChangeTo(name.to_string()),
                },
                "ls" => Instruction::List(
                    c.lines()
                        .skip(1)
                        .map(|line| {
                            let mut parts = line.split_whitespace();
                            let size = parts.next().unwrap();
                            if size == "dir" {
                                (None, parts.next().unwrap().to_string())
                            } else {
                                (
                                    Some(size.parse::<u64>().unwrap()),
                                    parts.next().unwrap().to_string(),
                                )
                            }
                        })
                        .collect(),
                ),
                other => panic!("invalid instruction '{}'", other),
            }
        })
        .collect()
}

struct Directory {
    name: String,
    files: HashMap<String, u64>,
    directories: HashMap<String, Directory>,
    cached_size: u64,
}

impl Directory {
    pub const TOTAL_SPACE: u64 = 70_000_000;
    pub const NECESSARY_SPACE: u64 = 30_000_000;

    pub fn new(name: String) -> Self {
        Self {
            name,
            files: Default::default(),
            directories: Default::default(),
            cached_size: 0,
        }
    }
    pub fn update_cached_size(&mut self) -> u64 {
        self.cached_size = self.files.values().sum::<u64>()
            + self
                .directories
                .values_mut()
                .map(|d| d.update_cached_size())
                .sum::<u64>();
        self.cached_size
    }
    pub fn get_directories_recursive(&self) -> Vec<&Directory> {
        let inner: Vec<Vec<&Directory>> = self
            .directories
            .values()
            .map(|d| d.get_directories_recursive())
            .collect();
        let mut directories = Vec::with_capacity(inner.iter().map(|d| d.len()).sum::<usize>() + 1);
        directories.push(self);
        for mut d in inner {
            directories.append(&mut d);
        }
        directories
    }
    pub fn get_smallest_directory_big_enough(&self) -> &Directory {
        let needed = Self::NECESSARY_SPACE - (Self::TOTAL_SPACE - self.cached_size);
        self.get_directories_recursive()
            .into_iter()
            .filter(|dir| dir.cached_size >= needed)
            .min_by_key(|dir| dir.cached_size)
            .unwrap()
    }
}

enum Instruction {
    ChangeToRoot,
    ChangeTo(String),
    ChangeToPrevious,
    List(Vec<(Option<u64>, String)>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let input = "$ cd /
$ ls
dir a
14848514 b.txt
8504156 c.dat
dir d
$ cd a
$ ls
dir e
29116 f
2557 g
62596 h.lst
$ cd e
$ ls
584 i
$ cd ..
$ cd ..
$ cd d
$ ls
4060174 j
8033020 d.log
5626152 d.ext
7214296 k";

        let dir = build_directories(input);
        assert_eq!(
            95437,
            dir.get_directories_recursive()
                .into_iter()
                .filter(|d| d.cached_size <= 100_000)
                .map(|d| d.cached_size)
                .sum::<u64>()
        );

        assert_eq!(
            24933642,
            dir.get_smallest_directory_big_enough().cached_size
        )
    }
}
