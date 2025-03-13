export class Deferred<T> {
  public readonly promise: Promise<T>;
  public accept!: (value: T) => void;
  public reject!: (error: any) => void;

  constructor() {
    this.promise = new Promise((accept, reject) => {
      this.accept = accept;
      this.reject = reject;
    });
  }
}
