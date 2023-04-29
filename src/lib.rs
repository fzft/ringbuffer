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
        let (mut pro, con) = db.split();
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
        let (mut pro, con) = db.split();
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
        let (mut pro, con) = db.split();
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
    fn test_rb_vacant_range() {
        let db = SharedRb::<u8, 5>::new();
        let (mut pro, con) = db.split();

        assert_eq!(pro.cap(), 5);
        assert_eq!(pro.count(), 0);

        assert_eq!(pro.vacant_slices().0.len(), 5);
        assert_eq!(pro.vacant_slices().1.len(), 0);
        assert_eq!(con.occupied_slices().0.len(), 0);
        assert_eq!(con.occupied_slices().1.len(), 0);

        // [tail, head, 0, 0, 0]
        // [1, 5] [0, 0]
        // [0, 1] [0, 0]
        let _ = pro.push(1);
        assert_eq!(pro.vacant_slices().0.len(), 4);
        assert_eq!(pro.vacant_slices().1.len(), 0);
        assert_eq!(con.occupied_slices().0.len(), 1);
        assert_eq!(con.occupied_slices().1.len(), 0);

        // [tail, 0, head, 0, 0]
        // [2, 5] [0, 0]
        // [0,2] [0,0]
        let _ = pro.push(1);
        assert_eq!(pro.vacant_slices().0.len(), 3);
        assert_eq!(pro.vacant_slices().1.len(), 0);
        assert_eq!(con.occupied_slices().0.len(), 2);
        assert_eq!(con.occupied_slices().1.len(), 0);

        // [0, tail, head, 0, 0]
        // [2,5] [0,1]
        // [1,2] [0,0]
        con.pop();
        assert_eq!(pro.vacant_slices().0.len(), 3);
        assert_eq!(pro.vacant_slices().1.len(), 1);
        assert_eq!(con.occupied_slices().0.len(), 1);
        assert_eq!(con.occupied_slices().1.len(), 0);

        // [0, 0, head-tail, 0, 0]
        // [2, 5] [0,2]
        // [0,0] [0,0]
        con.pop();
        assert_eq!(pro.vacant_slices().0.len(), 3);
        assert_eq!(pro.vacant_slices().1.len(), 2);
        assert_eq!(con.occupied_slices().0.len(), 0);
        assert_eq!(con.occupied_slices().1.len(), 0);


        // [0, 0, tail, head, 0]
        // [3, 5] [0,2]
        // [2, 3] [0,0]
        let _ = pro.push(1);
        assert_eq!(pro.vacant_slices().0.len(), 2);
        assert_eq!(pro.vacant_slices().1.len(), 2);
        assert_eq!(con.occupied_slices().0.len(), 1);
        assert_eq!(con.occupied_slices().1.len(), 0);

        // [0, 0, tail, 0, head]
        // [4, 5] [0,2]
        // [2,4] [0,0]
        let _ = pro.push(1);
        assert_eq!(pro.vacant_slices().0.len(), 1);
        assert_eq!(pro.vacant_slices().1.len(), 2);
        assert_eq!(con.occupied_slices().0.len(), 2);
        assert_eq!(con.occupied_slices().1.len(), 0);

        eprintln!("is_full:{}", pro.is_full());

        // [head, 0, tail, 0, 0]
        // [0, 2] [0, 0]
        // [2, 5]
        let _ = pro.push(1);
        assert_eq!(pro.vacant_slices().0.len(), 2);
        assert_eq!(pro.vacant_slices().1.len(), 0);
        assert_eq!(con.occupied_slices().0.len(), 3);
        assert_eq!(con.occupied_slices().1.len(), 0);
    }

    #[test]
    fn test_rb_slice_op() {
        let db = SharedRb::<u8, 10>::new();
        let (mut pro, con) = db.split();

        let v1 = vec![0, 1, 2, 3, 4];
        pro.push_slice(&v1);

        let mut v2: [u8; 3] = [0; 3];
        con.pop_slice(&mut v2);

        assert_eq!([0, 1, 2], v2)
    }

    #[test]
    fn test_rb_slice_op2() {
        let db = SharedRb::<u8, 10>::new();
        let (mut pro, con) = db.split();

        let v1 = vec![0, 1, 2, 3, 4];
        pro.push_slice(&v1);

        let mut v2: [u8; 6] = [0; 6];
        con.pop_slice(&mut v2);

        eprintln!("{:?}", v2)
    }

    #[test]
    fn test_rb_slice_op3() {
        let db = SharedRb::<u8, 10>::new();
        let (mut pro, con) = db.split();

        let _ = pro.push(1);
        let _ = pro.push(1);
        let _ = pro.push(1);
        let _ = pro.push(1);
        let _ = pro.push(1);
        let _ = pro.push(1);
        let _ = pro.push(1);
        let _ = pro.push(1);

        con.pop();
        con.pop();
        con.pop();
        con.pop();
        con.pop();
        con.pop();
        con.pop();
        con.pop();

        let v1 = vec![0, 1, 2, 3, 4];
        pro.push_slice(&v1);

        let mut v2: [u8; 6] = [0; 6];
        con.pop_slice(&mut v2);

        eprintln!("{:?}", v2)
    }

    #[test]
    fn test_rb_read_and_write() {
        let db = SharedRb::<u8, 1024>::new();
        let (mut pro, con) = db.split();
        let smsg = "The quick brown fox jumps over the lazy dog";
        let zero = [0];
        let mut bytes = smsg.as_bytes().chain(&zero[..]);
        let n = pro.read_from(&mut bytes).unwrap();


        eprintln!("{}", n)
    }

    #[test]
    fn test_rb_concurrent() {
        let db = SharedRb::<u8, 10>::new();
        let (mut pro, con) = db.split();

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
                        break;
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


