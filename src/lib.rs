pub mod recursive_borrow {
    pub struct DFS<
        'a,
        C,
        O: Copy,
        T: FnOnce(&mut C, O),
        U: FnMut(&mut C, O),
        V: FnOnce(&C) -> bool,
        W: FnOnce(&C) -> X,
        X: IntoIterator<Item = O>,
    > {
        status: &'a mut C,
        forward: T,
        back_tracking: U,
        check: V,
        get_all_ops: W,
        op: O,
    }
    impl<
            'a,
            C: std::fmt::Debug,
            O: Copy,
            T: Copy + FnOnce(&mut C, O),
            U: Copy + FnMut(&mut C, O),
            V: Copy + FnOnce(&C) -> bool,
            // W: FnOnce(&C) -> X,
            // move occurs because `self.get_all_ops` has type `W`, which does not implement the `Copy` trait
            W: Copy + FnOnce(&C) -> X,
            X: IntoIterator<Item = O>,
        > DFS<'a, C, O, T, U, V, W, X>
    {
        pub fn new(
            status: &'a mut C,
            forward: T,
            back_tracking: U,
            check: V,
            get_all_ops: W,
            op: O,
        ) -> Self {
            Self {
                status,
                forward,
                back_tracking,
                check,
                get_all_ops,
                op,
            }
        }
        pub fn dfs<'b>(&'b mut self)
        where
            'a: 'b,
        {
            if (self.check)(self.status) {
                return;
            }
            for op in (self.get_all_ops)(self.status) {
                let mut next = self.forward(op);
                next.op = op;
                next.dfs();
            }
        }
        pub fn dfs_early_stop<'b>(&'b mut self) -> bool
        where
            'a: 'b,
        {
            if (self.check)(self.status) {
                return true;
            }
            for op in (self.get_all_ops)(self.status) {
                let mut next = self.forward(op);
                next.op = op;
                if next.dfs_early_stop() {
                    return true;
                }
            }
            false
        }
        pub fn forward<'b: 'c, 'c>(&'b mut self, op: O) -> DFS<'b, C, O, T, U, V, W, X>
        where
            'b: 'c,
        {
            (self.forward)(self.status, op);
            DFS {
                status: self.status,
                forward: self.forward,
                back_tracking: self.back_tracking,
                check: self.check,
                get_all_ops: self.get_all_ops,
                op,
            }
        }
    }
    impl<
            'a,
            C,
            O: Copy,
            T: FnOnce(&mut C, O),
            U: FnMut(&mut C, O),
            V: FnOnce(&C) -> bool,
            W: FnOnce(&C) -> X,
            X: IntoIterator<Item = O>,
        > Drop for DFS<'a, C, O, T, U, V, W, X>
    {
        fn drop(&mut self) {
            (self.back_tracking)(self.status, self.op)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        const N: usize = 3;
        {
            let mut s = [0; 2 * N + 1];
            s[0] = N as i32;
            let mut a = recursive_borrow::DFS::new(
                &mut s,
                |s: &mut [i32; 2 * N + 1], op: (i32, i32)| {
                    s[op.1 as usize] = op.0;
                    s[(1 + op.0 + op.1) as usize] = op.0;
                    s[0] -= 1;
                },
                |s: &mut [i32; 2 * N + 1], op: (i32, i32)| {
                    s[op.1 as usize] = 0;
                    s[(1 + op.0 + op.1) as usize] = 0;
                    s[0] += 1
                },
                |s: &[i32; 2 * N + 1]| {
                    if s[0] == 0 {
                        eprintln!("{s:?}");
                        true
                    } else {
                        false
                    }
                },
                |s: &[i32; 2 * N + 1]| {
                    let a = s[0];
                    (1..2 * N as i32 - a).filter_map(move |x| {
                        if s[x as usize] + s[(1 + a + x) as usize] == 0 {
                            Some((a, x))
                        } else {
                            None
                        }
                    }).collect::<Vec<_>>()
                },
                (1, 0),
            );
            a.dfs();
            a.dfs_early_stop();
        }
    }
}
