pub fn split_minutes(total_minutes: u32) -> (u32, u32) {
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    (hours, minutes)
}