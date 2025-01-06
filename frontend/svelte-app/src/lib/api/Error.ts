enum ValidationIssue {
  Invalid,
  Required,
  TooWeak
}

export type ValidationInfo = {
  field: string;
  issue: ValidationIssue;
};

export enum ErrorCode {
  ValidationError
}

export type ErrorResponse = {
  code: string;
  message: string;
  data: {
    ValidationInfo: Array<ValidationInfo>;
  };
};
