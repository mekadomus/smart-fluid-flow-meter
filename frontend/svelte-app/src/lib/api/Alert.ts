export enum AlertType {
  ConstantFlow = 'ConstantFlow',
  NotReporting = 'NotReporting'
}

export type Alert = {
  alert_type: AlertType;
};
