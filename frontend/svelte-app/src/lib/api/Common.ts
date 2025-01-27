export const PAGE_SIZE = 25;

export type Pagination = {
  has_more: boolean;
  has_less: boolean;
};

export type PaginatedResponse<T> = {
  items: T[];
  pagination: Pagination;
};
