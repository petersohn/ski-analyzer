import { indexSkiArea, RawSkiArea, SkiArea } from "@/types/skiArea";
import { RawTrack, TrackConverter } from "@/types/track";
import { computed, Injectable, signal } from "@angular/core";
import { Event, listen, UnlistenFn } from "@tauri-apps/api/event";
import { ActionsService } from "./actions.service";
import { TasksService } from "./tasks.service";
import { TaskResult } from "@/types/task";

@Injectable({ providedIn: "root" })
export class EventsService {
  public activeSkiArea = signal<SkiArea | null>(null);
  public activeTrack = computed(() => {
    const skiArea = this.activeSkiArea();
    if (!skiArea) {
      return null;
    }
    const track = this.activeRawTrack();
    if (!track) {
      return null;
    }
    return new TrackConverter(skiArea).convertTrack(track);
  });
  public isInitialized = signal(false);

  private activeRawTrack = signal<RawTrack | null>(null);
  private unlistens: UnlistenFn[] = [];

  constructor(
    private readonly actionsService: ActionsService,
    private readonly tasksService: TasksService,
  ) {}

  public async initEvents() {
    this.unlistens.push(
      await listen("active_ski_area_changed", (event: Event<RawSkiArea>) => {
        this.activeSkiArea.set(indexSkiArea(event.payload));
      }),
    );
    this.unlistens.push(
      await listen("active_route_changed", (event: Event<RawTrack>) => {
        this.activeRawTrack.set(event.payload);
      }),
    );
    this.unlistens.push(
      await listen("task_finished", (event: Event<TaskResult>) => {
        this.tasksService.acceptTask(event.payload.task_id, event.payload.data);
      }),
    );
    this.unlistens.push(
      await listen("task_failed", (event: Event<TaskResult>) => {
        this.tasksService.rejectTask(event.payload.task_id, event.payload.data);
      }),
    );

    const skiArea = await this.actionsService.getActiveSkiArea();
    this.activeSkiArea.set(skiArea ? indexSkiArea(skiArea) : null);

    const route = await this.actionsService.getActiveRoute();
    this.activeRawTrack.set(route ?? null);

    this.isInitialized.set(true);
  }

  public deinitEvents() {
    for (const u of this.unlistens) {
      u();
    }
  }
}
