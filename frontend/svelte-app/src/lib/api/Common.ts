export type Pagination = {
  has_more: boolean;
  has_less: boolean;
};

export type PaginatedResponse<T> = {
  items: T[];
  pagination: Pagination;
};
