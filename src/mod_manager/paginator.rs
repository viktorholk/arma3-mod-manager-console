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
