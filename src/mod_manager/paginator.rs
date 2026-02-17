#[derive(Debug)]
pub struct Paginator<T> {
    items: Vec<T>,
    pub page_size: usize,
    pub current_page: usize,
}

impl<T> Paginator<T> {
    pub fn new(items: Vec<T>, page_size: usize) -> Self {
        Paginator {
            items,
            page_size,
            current_page: 0,
        }
    }

    pub fn total_pages(&self) -> usize {
        self.items.len().div_ceil(self.page_size)
    }

    pub fn all_items(&self) -> &[T] {
        &self.items
    }

    pub fn all_items_mut(&mut self) -> &mut [T] {
        &mut self.items
    }

    pub fn current_page_items(&self) -> &[T] {
        let start = self.current_page * self.page_size;
        let end = usize::min(start + self.page_size, self.items.len());
        &self.items[start..end]
    }

    pub fn next_page(&mut self) {
        if self.current_page + 1 < self.total_pages() {
            self.current_page += 1;
        }
    }

    pub fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
        }
    }

    pub fn filter<F>(&self, predicate: F) -> Vec<&T>
    where
        F: Fn(&T) -> bool,
    {
        self.items.iter().filter(|item| predicate(item)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_paginator(count: usize, page_size: usize) -> Paginator<i32> {
        Paginator::new((0..count as i32).collect(), page_size)
    }

    #[test]
    fn total_pages_exact_fit() {
        let p = make_paginator(10, 5);
        assert_eq!(p.total_pages(), 2);
    }

    #[test]
    fn total_pages_partial_last_page() {
        let p = make_paginator(11, 5);
        assert_eq!(p.total_pages(), 3);
    }

    #[test]
    fn total_pages_empty() {
        let p = make_paginator(0, 5);
        assert_eq!(p.total_pages(), 0);
    }

    #[test]
    fn total_pages_fewer_than_page_size() {
        let p = make_paginator(3, 10);
        assert_eq!(p.total_pages(), 1);
    }

    #[test]
    fn current_page_items_first_page() {
        let p = make_paginator(7, 3);
        assert_eq!(p.current_page_items(), &[0, 1, 2]);
    }

    #[test]
    fn current_page_items_last_partial_page() {
        let mut p = make_paginator(7, 3);
        p.current_page = 2; // third page: only item 6
        assert_eq!(p.current_page_items(), &[6]);
    }

    #[test]
    fn next_page_advances() {
        let mut p = make_paginator(10, 3);
        assert_eq!(p.current_page, 0);
        p.next_page();
        assert_eq!(p.current_page, 1);
        assert_eq!(p.current_page_items(), &[3, 4, 5]);
    }

    #[test]
    fn next_page_clamps_at_last() {
        let mut p = make_paginator(5, 5); // 1 page total
        p.next_page();
        assert_eq!(p.current_page, 0);
    }

    #[test]
    fn prev_page_clamps_at_zero() {
        let mut p = make_paginator(10, 5);
        p.prev_page();
        assert_eq!(p.current_page, 0);
    }

    #[test]
    fn prev_page_goes_back() {
        let mut p = make_paginator(10, 3);
        p.next_page();
        p.next_page();
        assert_eq!(p.current_page, 2);
        p.prev_page();
        assert_eq!(p.current_page, 1);
    }

    #[test]
    fn filter_returns_matching() {
        let p = make_paginator(6, 10);
        let evens = p.filter(|x| *x % 2 == 0);
        assert_eq!(evens.len(), 3);
        assert_eq!(*evens[0], 0);
        assert_eq!(*evens[1], 2);
        assert_eq!(*evens[2], 4);
    }

    #[test]
    fn filter_returns_empty_when_no_match() {
        let p = make_paginator(3, 10);
        let result = p.filter(|x| *x > 100);
        assert!(result.is_empty());
    }
}
