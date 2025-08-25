pub fn decode(input: &String) -> Vec<u8> {
    input
        .chars()
        .map(|c| c as u32 as u8) // convert char -> u32 -> u8 (truncates like TypeScript)
        .collect()
}

pub fn encode(input: &Vec<u8>) -> String {
    input.iter().map(|&b| b as char).collect()
}
