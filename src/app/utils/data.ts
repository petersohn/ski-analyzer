export function indexData<T>(data: { [id: string]: T }): Map<string, T> {
  return new Map(Object.keys(data).map((id) => [id, data[id]]));
}

export function indexAndConvertData<T1, T2>(
  data: { [id: string]: T1 },
  convert: (x: T1) => T2,
): Map<string, T2> {
  return new Map(Object.keys(data).map((id) => [id, convert(data[id])]));
}
