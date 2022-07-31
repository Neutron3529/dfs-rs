use std::ops::{Index, IndexMut};
pub trait Op<T> where Self:Sized{
    fn r#do(&mut self, status: &mut T);
    fn undo(&mut self, status: &mut T);
    fn chain<W:Op<T>>(self,other:W)->Chain<Self,W,T>{Chain(self,other,[])}
}
impl<T> Op<T> for (){
    fn r#do(&mut self, _: &mut T){}
    fn undo(&mut self, _: &mut T){}
}
#[derive(Copy, Clone)]
pub struct Add<I, V>(pub I, pub V);
impl<T: IndexMut<I>, I: Copy, V: Copy> Op<T> for Add<I, V>
where
    <T as Index<I>>::Output: std::ops::SubAssign<V> + std::ops::AddAssign<V>,
{
    #[inline(always)]
    fn r#do(&mut self, status: &mut T) {
        status[self.0] += self.1
    }
    #[inline(always)]
    fn undo(&mut self, status: &mut T) {
        status[self.0] -= self.1
    }
}
#[derive(Copy, Clone)]
pub struct Assign<I, V>(pub I, pub V);
impl<T:Index<I, Output = V>+IndexMut<I>, I: Copy, V: Copy+Sized> Op<T> for Assign<I, V> where  {
    #[inline(always)]
    fn r#do(&mut self, status: &mut T) {
        std::mem::swap(&mut status[self.0] , &mut self.1)
    }
    #[inline(always)]
    fn undo(&mut self, status: &mut T) {
        std::mem::swap(&mut status[self.0] , &mut self.1)
    }
}



#[derive(Copy, Clone)]
pub struct Chain<I:Op<T>, V:Op<T>,T>(I, V,[T;0]);
impl<T> Chain<(),(),T>{
    pub fn new()->Chain<(),(),T>{
        Chain((),(),[])
    }
}
impl<I:Op<T>, V:Op<T>,T> Chain<I, V,T>{
    pub fn chain<W:Op<T>>(self,op:W)->Chain<Chain<I, V,T>,W,T>{
        Chain(self,op,[])
    }
}
impl<I:Op<T>, V:Op<T>,T> Op<T> for Chain<I,V,T>{
    #[inline(always)]
    fn r#do(&mut self, status: &mut T){
        self.0.r#do(status);
        self.1.r#do(status);
    }
    #[inline(always)]
    fn undo(&mut self, status: &mut T){
        self.1.undo(status);
        self.0.undo(status);
    }
}
pub struct DFS<
    'a,
    I,
    C: Index<I,Output:Sized> + IndexMut<I>,
    O: Op<C>,
    V: FnOnce(&C) -> bool,
    W: FnOnce(&C) -> X,
    X: IntoIterator<Item = O>,
> where <C as Index<I>>::Output:Sized{
    status: &'a mut C,
    check: V,
    get_all_ops: W,
    op: O,
    _i:[I;0],
}
impl<
        'a,
        I,
        C: Index<I,Output:Sized> + IndexMut<I> + std::fmt::Debug,
        O: Op<C>,
        V: Copy + FnOnce(&C) -> bool, // check success
        W: Copy + FnOnce(&C) -> X,
        X: IntoIterator<Item = O>,
    > DFS<'a,I, C, O, V, W, X>
{
    pub fn new(
        status: &'a mut C,
        check: V,
        get_all_ops: W,
        op: O,
    ) -> Self {
        Self {
            status,
            check,
            get_all_ops,
            op,_i:[]
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
            if next.dfs_early_stop() {
                return true;
            }
        }
        false
    }
    pub fn forward<'b: 'c, 'c>(&'b mut self, mut op: O) -> DFS<'b,I, C, O, V, W, X>
    where
        'b: 'c,
    {
        op.r#do(self.status);
        DFS {
            status: self.status,
            check: self.check,
            get_all_ops: self.get_all_ops,
            op,_i:[]
        }
    }
}
impl<
        'a,
        I,
        C: Index<I,Output:Sized> + IndexMut<I>,
        O: Op<C>,
        V: FnOnce(&C) -> bool,
        W: FnOnce(&C) -> X,
        X: IntoIterator<Item = O>,
    > Drop for DFS<'a,I, C, O, V, W, X>
{
    fn drop(&mut self) {
        self.op.undo(self.status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        const N: usize = 3;
        {
            let mut s = [0i32; 2 * N + 1];
            s[0] = N as i32;
            let mut a = DFS::<usize,_,_,_,_,_>::new(
                &mut s,
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
                    (1..2 * N as i32 - a)
                        .filter_map(move |x| {
                            if s[x as usize] + s[(1 + a + x) as usize] == 0 {
                                Some(Chain::new().chain(Add(x as usize,a)).chain(Add((1+a+x) as usize,a)).chain(Add(0,-1)))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                },
                Chain::new().chain(Add(0,1)).chain(Add(0,1)).chain(Add(0,-2)),
            );
            a.dfs();
            a.dfs_early_stop();
        }
    }

    #[test]
    fn op_works(){
        let mut a=[0,1,2,3,4,5];
        let mut b=Add(0,1);
        b.r#do(&mut a);
        assert!(a==[1,1,2,3,4,5]);
        #[derive(Copy, Clone)]
        struct Swap<I>(I, I);
        impl Op<[i32;6]> for Swap<usize> {
            #[inline(always)]
            fn r#do(&mut self, status: &mut [i32;6]) {
                status.swap(self.0,self.1)
            }
            #[inline(always)]
            fn undo(&mut self, status: &mut [i32;6]) {
                status.swap(self.0,self.1)
            }
        }
        let mut c=Swap(4,5);
        c.r#do(&mut a);
        assert!(a==[1,1,2,3,5,4]);
        let mut d=Assign(5,8);
        d.r#do(&mut a);
        assert!(a==[1,1,2,3,5,8]);
        let mut chain=b.chain(c).chain(d);
        chain.undo(&mut a);
        assert!(a==[0,1,2,3,4,5]);
    }
}
