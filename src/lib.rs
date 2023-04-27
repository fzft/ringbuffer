#![feature(slice_from_ptr_range)]

mod rb;
mod base;
mod producer;
mod consumer;
mod storage;


#[cfg(test)]
mod tests {
    use std::io::Read;
    use std::thread;
    use std::time::Duration;

    use crate::base::RbBase;
    use crate::storage::SharedRb;

    use super::*;

    #[test]
    fn test_rb_base() {
        let db = SharedRb::<u32, 10>::new();
        let (mut pro, mut con) = db.split();
        let _ = pro.push(34);
        let ele = con.pop();
        assert_eq!(34, ele.unwrap())
    }

    #[test]
    fn test_rb_base_empty() {
        let db = SharedRb::<u32, 10>::new();
        assert_eq!(db.is_empty(), true);
    }

    #[test]
    fn test_rb_base_full() {
        let db = SharedRb::<u32, 3>::new();
        let (mut pro, _) = db.split();
        let _ = pro.push(1);
        let _ = pro.push(2);
        let _ = pro.push(3);

        assert_eq!(pro.is_full(), true);
    }

    #[test]
    fn test_rb_base_full_push() {
        let db = SharedRb::<u32, 3>::new();
        let (mut pro, mut con) = db.split();
        let _ = pro.push(1);
        let _ = pro.push(2);
        let result = pro.push(3);
        assert!(result.is_ok());
        let result = pro.push(4);
        assert!(result.is_err());

        con.pop();
        assert!(!pro.is_full());

        let result = pro.push(5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rb_base_empty_pop() {
        let db = SharedRb::<u32, 3>::new();
        let (mut pro, mut con) = db.split();
        let _ = pro.push(1);
        let _ = pro.push(2);

        let result = con.pop();
        assert_eq!(1, result.unwrap());
        let result = con.pop();
        assert_eq!(2, result.unwrap());

        let result = con.pop();
        assert_eq!(None, result);

        let _ = pro.push(3);

        let result = con.pop();
        assert_eq!(3, result.unwrap());
    }

    #[test]
    fn test_rb_concurrent() {
        let db = SharedRb::<u8, 10>::new();
        let (mut pro, mut con) = db.split();

        let smsg = "The quick brown fox jumps over the lazy dog";

        let pjh = thread::spawn(move || {
            dbg!("sending message");
            let zero = [0];
            let mut bytes = smsg.as_bytes().chain(&zero[..]);
            loop {
                if pro.is_full() {
                    dbg!("buffer is full, waiting");
                    thread::sleep(Duration::from_millis(100));
                } else {
                    let n = pro.read_from(&mut bytes).unwrap();
                    if n == 0 {
                        break;
                    }
                    dbg!("-> {} bytes sent", n);
                }
            }
            dbg!("-> message sent");
        });


        let cjh = thread::spawn(move || {
            dbg!("recv message");
            let mut bytes = Vec::<u8>::new();
            loop {
                if con.is_empty() {
                    if bytes.ends_with(&[0]) {
                        break
                    }
                    dbg!("buffer is empty, waiting");
                    thread::sleep(Duration::from_millis(100));
                } else {
                    let n = con.write_into(&mut bytes).unwrap();
                    dbg!("<- {} bytes received", n);
                }
            }

            let msg = String::from_utf8(bytes).unwrap();
            dbg!("<- message received: '{}'", msg);
        });


        pjh.join().unwrap();
        cjh.join().unwrap();
    }
}


