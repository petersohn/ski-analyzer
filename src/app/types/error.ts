import { Rect } from "./geo";

export type ErrorType =
  | "InputError"
  | "OSMError"
  | "LogicError"
  | "NoSkiAreaAtLocation"
  | "IoError"
  | "NetworkError"
  | "FormatError"
  | "Cancelled"
  | "UnknownError";

export type ErrorDetails = Rect;

export type Error = {
  type: ErrorType;
  details?: ErrorDetails;
  msg: string;
};
