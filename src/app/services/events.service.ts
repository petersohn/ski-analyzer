import { indexSkiArea, RawSkiArea, SkiArea } from "@/types/skiArea";
import { RawTrack, TrackConverter } from "@/types/track";
import { computed, Injectable, signal } from "@angular/core";
import { Event, listen, UnlistenFn } from "@tauri-apps/api/event";
import { ActionsService } from "./actions.service";

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

  private activeRawTrack = signal<RawTrack | null>(null);
  private unlistens: UnlistenFn[] = [];

  constructor(private readonly actionsService: ActionsService) {}

  public async initEvents() {
    this.unlistens.push(
      await listen("active_ski_area_changed", (event: Event<RawSkiArea>) =>
        this.activeSkiArea.set(indexSkiArea(event.payload)),
      ),
    );
    this.unlistens.push(
      await listen("active_route_changed", (event: Event<RawTrack>) =>
        this.activeRawTrack.set(event.payload),
      ),
    );

    const skiArea = await this.actionsService.getActiveSkiArea();
    this.activeSkiArea.set(skiArea ? indexSkiArea(skiArea) : null);

    const route = await this.actionsService.getActiveRoute();
    this.activeRawTrack.set(route ?? null);
  }

  public deinitEvents() {
    for (const u of this.unlistens) {
      u();
    }
  }
}
