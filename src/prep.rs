use std::fs;

pub fn fs_prep(runtime_dir: &str) -> String {
    let mut list: Vec<usize> = fs::read_dir(runtime_dir)
        .unwrap()
        .filter(|result| result.is_ok())
        .map(|result| result.unwrap())
        .filter(|dir_entry| dir_entry.file_type().is_ok())
        .filter(|dir_entry| dir_entry.file_type().unwrap().is_dir())
        .map(|dir_entry| dir_entry.file_name())
        .map(|file_name| file_name.into_string())
        .filter(|result| result.is_ok())
        .map(|string| string.unwrap().parse::<usize>())
        .filter(|result| result.is_ok())
        .map(|result| result.unwrap())
        .collect();

    list.sort_unstable();

    let mut dir_num: usize = list.len() + 1;
    for index in 1..=list.len() {
        if index != list[index - 1] {
            dir_num = index;
            break;
        }
    }

    let req_dir = format!("{}/{}", runtime_dir, dir_num);
    fs::create_dir_all(&req_dir).unwrap();

    req_dir.to_string()
}

pub fn net_prep(veth_name: &str, pid: i32) {}
