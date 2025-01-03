import {
  Component,
  AfterViewInit,
  OnDestroy,
  ElementRef,
  ViewChild,
  effect,
  EffectRef,
} from "@angular/core";
import { MapService } from "@/services/map.service";
import { ActionsService } from "@/services/actions.service";
import { EventsService } from "@/services/events.service";
import { indexSkiArea } from "@/types/skiArea";
import { TrackConverter } from "@/types/track";

@Component({
  selector: "map",
  standalone: true,
  imports: [],
  templateUrl: "./map.component.html",
  styleUrls: ["./map.component.css"],
})
export class MapComponent implements AfterViewInit, OnDestroy {
  @ViewChild("map")
  public mapElement!: ElementRef<HTMLElement>;

  private effects: EffectRef[] = [];

  constructor(
    private readonly mapService: MapService,
    private readonly actionsService: ActionsService,
    private readonly eventsService: EventsService,
  ) {
    this.effects.push(
      effect(() => {
        const mapConfig = this.mapService.mapConfig();
        if (!!mapConfig) {
          this.actionsService.saveMapConfig(mapConfig);
        }
      }),
    );
    this.effects.push(
      effect(() => {
        const skiArea = this.eventsService.activeSkiArea();
        if (!skiArea) {
          this.mapService.unloadSkiArea();
        } else {
          this.mapService.loadSkiArea(skiArea, true);
        }
      }),
    );

    this.effects.push(
      effect(() => {
        const track = this.eventsService.activeTrack();
        if (!track) {
          this.mapService.unloadTrack();
        } else {
          this.mapService.loadTrack(track);
        }
      }),
    );
  }

  public async ngAfterViewInit() {
    await this.mapService.createMap(this.mapElement.nativeElement);

    const mapConfig = await this.actionsService.getMapConfig();
    if (!!mapConfig) {
      this.mapService.setMapConfig(mapConfig);
    }

    const rawSkiArea = await this.actionsService.getActiveSkiArea();
    if (!!rawSkiArea) {
      const skiArea = indexSkiArea(rawSkiArea);
      this.mapService.loadSkiArea(skiArea, !mapConfig);
      const rawRoute = await this.actionsService.getActiveRoute();
      if (!!rawRoute) {
        this.mapService.loadTrack(
          new TrackConverter(skiArea).convertTrack(rawRoute),
        );
      }
    }
  }

  public ngOnDestroy() {
    for (const e of this.effects) {
      e.destroy();
    }

    this.mapService.removeMap();
  }
}
