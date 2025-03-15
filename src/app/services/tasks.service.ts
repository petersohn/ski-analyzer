import { Injectable, signal } from "@angular/core";
import { Deferred } from "@/utils/deferred";

@Injectable({ providedIn: "root" })
export class TasksService {
  private tasks: Map<number, Deferred<any>> = new Map();
  public hasTask = signal(false);

  public addTask(id: number): Promise<any> {
    const deferred = new Deferred();
    this.tasks.set(id, deferred);
    this.calculateHasTask();
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
    this.calculateHasTask();
  }

  public rejectTask(id: number, value: any): void {
    const deferred = this.tasks.get(id);
    if (!deferred) {
      console.error(`Unknown task id: ${id}`);
      return;
    }
    deferred.reject(value);
    this.tasks.delete(id);
    this.calculateHasTask();
  }

  private calculateHasTask() {
    this.hasTask.set(this.tasks.size !== 0);
  }
}
