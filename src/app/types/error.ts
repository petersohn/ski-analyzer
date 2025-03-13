import { Rect } from "./geo";

export type ErrorType =
  | "InputError"
  | "OSMError"
  | "LogicError"
  | "ExternalError"
  | "NoSkiAreaAtLocation"
  | "Cancelled";

export type ErrorDetails = Rect;

export type Error = {
  type: ErrorType;
  details?: ErrorDetails;
  msg: string;
};
