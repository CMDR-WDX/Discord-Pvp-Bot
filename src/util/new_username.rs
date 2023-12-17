pub fn convert_tag_to_username(username: String) -> String {
    let username = username;
   
    let segment =  username.split("#").find(|_x| true);
    match segment {
        Some(e) => return e.to_string(),
        None => return username
    }
}