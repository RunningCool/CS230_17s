extern crate hyper;
extern crate url;
use self::hyper::Client;
use self::hyper::client::RequestBuilder;
use self::hyper::header::Headers;
use self::hyper::status::StatusCode;
use self::hyper::client::response::Response as HpResp;
use self::hyper::error::Error as HpErr;
// use self::hyper::client::response::Response;
use self::url::Url;
use std::sync::mpsc::channel;
use std::thread;
use std::io::{Read, Error as ReadErr};
use std::io::ErrorKind;
use std::time::Duration;
use std::sync::Arc;
use engine::Crawler;

/// The struct representing the result of a http request.
// #[derive(Debug)]
// pub enum Error {
//     /// Status code is ok, however the program fails to convert the received stream to a vector of bytes.
//     ReadError(Url, ReadErr),
//     /// The status is not ok.
//     BadStatus(Url, StatusCode),
//     /// The program fails to establish a connection to the remote server.
//     ConnectionFailed(Url),
//     /// Timed out.
//     TimedOut(Url),
// }

// impl fmt::Display for UrlState {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match *self {
//             UrlState::BadStatus(ref url, ref status) => format!("✘ {} ({})", url, status).fmt(f),
//             UrlState::ConnectionFailed(ref url) => format!("✘ {} (connection failed)", url).fmt(f),
//             UrlState::TimedOut(ref url) => format!("✘ {} (timed out)", url).fmt(f),
//             UrlState::ReadError(ref url, ref err) => format!("✘ {} ({})", url, err).fmt(f),
//         }
//     }
// }

pub enum Error{
    /// The status code is not Ok.
    BadStatus(HpResp),
    /// Special read error timeout.
    TimedOut(HpResp),
    /// Other read error except for timeout.
    ReadError(HpResp, ReadErr),
    /// Errors that can occur parsing HTTP streams.
    BadRequest(HpErr),
}

#[derive(Clone)]
pub enum Method {
    Get,
    Post,
}

pub struct Response {
    pub url: Url,
    pub headers: Headers,
    pub body: Vec<u8>,//TODO: use a better linear container.
}

#[derive(Clone)]
pub struct RequestContent{
    pub url: Url,
    pub method: Method,
    pub body: Option<String>,
}
pub struct Request
{
    pub content: RequestContent,
    pub client: Client,
}

impl Request {
    pub fn download(self)-> Result<Response, Error> {
        let url = self.content.url.clone();
        let mut client: RequestBuilder;
        match self.content.method {
            Method::Get => {
                client = self.client.get(url);
            },
            Method::Post => {
                client = self.client.post(url);
            },
        }
        if let Some(ref body) = self.content.body{
            client = client.body(body);
        }
        let response = client.send();
        match response{
            Ok(mut response) => {
                if let StatusCode::Ok = response.status{
                    let mut buffer = vec![];
                    match response.read_to_end(&mut buffer){
                        Ok(_) => {
                            Ok(Response{
                                url: response.url.clone(),
                                headers: response.headers.clone(),
                                body: buffer,
                            })
                        }
                        Err(e) => {
                            match e.kind(){
                                ErrorKind::TimedOut => {
                                    Err(Error::TimedOut(response))
                                }
                                _ => {
                                    Err(Error::ReadError(response, e))
                                }
                            }
                        }
                    }
                }
                else{
                    Err(Error::BadStatus(response))
                }
            },
            Err(e) => Err(Error::BadRequest(e))
        }
    //     let url = self.url.clone();
    //     let url_ = url.clone();
    //     let (tx, rx) = channel();
    //     let tx_ = tx.clone();
    //
    //     thread::spawn(move || {
    //         let mut client: RequestBuilder;
    //         match self.method {
    //             Method::Get => {
    //                 client = self.client.get(url);
    //             },
    //             Method::Post => {
    //                 client = self.client.post(url);
    //             },
    //         }
    //         if let Some(ref body) = self.body{
    //             client = client.body(body);
    //         }
    //         let response = client.send();
    //         let _ = tx.send(
    //             match response{
    //                 Ok(mut response) => {
    //                     if let StatusCode::Ok = response.status{
    //                         let mut buffer = vec![];
    //                         match response.read_to_end(&mut buffer){
    //                             Ok(_) => {
    //                                 Ok(Response{
    //                                     url: response.url.clone(),
    //                                     headers: response.headers.clone(),
    //                                     body: buffer,
    //                                 })
    //                             },
    //                             Err(err) => {
    //                                 Err(Error::ReadError(response.url.clone(), err))
    //                             },
    //                         }
    //                     }
    //                     else {
    //                         Err(Error::BadStatus(response.url.clone(), response.status))
    //                     }
    //                 },
    //                 Err(err) => Err(Error::ConnectionFailed(self.url)),
    //             }
    //         );
    //     });
    //
    //     thread::spawn(move || {
    //         thread::sleep(Duration::from_secs(20));
    //         let _ = tx_.send(Err(Error::TimedOut(url_)));
    //     });
    //     rx.recv().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_download() {
        let url = Url::parse("http://www.baidu.com").unwrap();
        let client = Client::new();
        let request:Request = Request{
            url: url,
            method: Method::Get,
            body: None,
            client: client,
        };
        request.download().unwrap();
    }
}
