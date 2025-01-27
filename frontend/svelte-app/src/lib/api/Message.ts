export enum MessageType {
  Success = 'success',
  Warning = 'warning',
  Error = 'error'
}

export type Message = {
  type: MessageType;
  text: string;
};
