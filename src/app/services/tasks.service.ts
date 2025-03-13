import { Deferred } from "@/utils/deferred";
import { Injectable } from "@angular/core";

@Injectable({ providedIn: "root" })
export class TasksService {
  private tasks: Map<number, Deferred<any>> = new Map();

  public addTask(id: number): Promise<any> {
    const deferred = new Deferred();
    this.tasks.set(id, deferred);
    return deferred.promise;
  }

  public acceptTask(id: number, value: any): void {
    const deferred = this.tasks.get(id);
    if (!deferred) {
      console.error(`Unknown task id: ${id}`);
      return;
    }
    deferred.accept(value);
    this.tasks.delete(id);
  }

  public rejectTask(id: number, value: any): void {
    const deferred = this.tasks.get(id);
    if (!deferred) {
      console.error(`Unknown task id: ${id}`);
      return;
    }
    deferred.reject(value);
    this.tasks.delete(id);
  }
}
