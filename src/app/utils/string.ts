export function filterString(input: string, search: string): boolean {
  if (search.length == 0) {
    return true;
  }

  if (search.length > input.length) {
    return false;
  }

  for (let i = 0; i <= input.length - search.length; ++i) {
    if (
      input
        .slice(i, i + search.length)
        .localeCompare(search, undefined, { sensitivity: "base" }) == 0
    ) {
      return true;
    }
  }
  return false;
}
