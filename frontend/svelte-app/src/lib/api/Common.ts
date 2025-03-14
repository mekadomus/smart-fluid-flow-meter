export const PAGE_SIZE = 25;

export type Pagination = {
  has_more: boolean;
  has_less: boolean;
};

export type PaginatedResponse<T> = {
  items: T[];
  pagination: Pagination;
};

enum SeriesGranularity {
  Hour,
  Day,
  Month
}

export type SeriesItem = {
  period_start: Date;
  // We use string so we have flexibility about the type
  value: string;
};

export type Series = {
  granularity: SeriesGranularity;
  items: [SeriesItem];
};
