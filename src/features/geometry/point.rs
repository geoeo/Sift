use std::cmp::{Ordering, Eq,PartialOrd, PartialEq, Ord};


#[derive(Debug,Clone,Copy,Eq)]
pub struct Point<T> where T: PartialOrd + PartialEq {
    pub x: T,
    pub y: T
}

impl<T> PartialEq for Point<T> where T: PartialOrd + Eq {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y== other.y
    }
}

impl<T> Ord for Point<T> where T: PartialOrd + Eq {
    fn cmp(&self, other: &Self) -> Ordering {
        let x_cmp = self.x.partial_cmp(&other.x);
        let y_cmp = self.y.partial_cmp(&other.y);
        
        match (x_cmp,y_cmp) {
            (Some(Ordering::Less),_) => Ordering::Less,
            (Some(Ordering::Greater),_) => Ordering::Greater,
            (Some(Ordering::Equal),Some(Ordering::Less)) => Ordering::Less,
            (Some(Ordering::Equal),Some(Ordering::Greater)) => Ordering::Greater,
            (Some(Ordering::Equal),Some(Ordering::Equal)) => Ordering::Equal,
            (None,_) => Ordering::Equal,
            (_,None) => Ordering::Equal
        }
    }
}

impl<T> PartialOrd for Point<T>  where T: PartialOrd + Eq {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
