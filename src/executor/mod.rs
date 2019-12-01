use crate::parser::*;

#[cfg(test)]
mod tests;

//use boa::exec;
//use std::io::Lines;
//use std::{env, fs::read_to_string, process::exit};

struct Error;

pub async fn execute() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let buffer = "\
GET https://httpbin.org/get
Accept: */*

    ";
    //    dbg!(exec(&buffer));
    let script = "\
var config = {
    stuff: function () {
        console.log('yep');
    }
}
config.stuff();
return 'a';
    ";

    let result = parse(buffer)?;


//    dbg!(result);

//    let mut res = surf::get("https://httpbin.org/get").await?;
    //    dbg!(exec(script));
//    dbg!(res.body_string().await?);
    Ok(())
}



