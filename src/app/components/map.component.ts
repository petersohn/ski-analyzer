import {
  Component,
  AfterViewInit,
  OnDestroy,
  ElementRef,
  ViewChild,
  effect,
  EffectRef,
  ChangeDetectionStrategy,
} from "@angular/core";
import { MapService } from "@/services/map.service";
import { ActionsService } from "@/services/actions.service";
import { EventsService } from "@/services/events.service";
import { SkiArea } from "@/types/skiArea";

@Component({
  selector: "map",
  imports: [],
  templateUrl: "./map.component.html",
  styleUrls: ["./map.component.scss"],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MapComponent implements AfterViewInit, OnDestroy {
  @ViewChild("map")
  public mapElement!: ElementRef<HTMLElement>;

  private effects: EffectRef[] = [];
  private currentSkiArea: SkiArea | null | undefined;

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
        const isInitialized = this.eventsService.isInitialized();

        if (
          !this.mapService.isInitialized() ||
          skiArea === this.currentSkiArea
        ) {
          return;
        }

        this.currentSkiArea = skiArea;

        setTimeout(() => {
          if (!skiArea) {
            this.mapService.unloadSkiArea();
          } else {
            this.mapService.loadSkiArea(skiArea, isInitialized);
          }
        }, 0);
      }),
    );

    this.effects.push(
      effect(() => {
        const track = this.eventsService.activeTrack();

        if (!this.mapService.isInitialized()) {
          return;
        }

        setTimeout(() => {
          if (!track) {
            this.mapService.unloadTrack();
          } else {
            this.mapService.loadTrack(track);
          }
        });
      }),
    );
  }

  public async ngAfterViewInit() {
    await this.mapService.createMap(this.mapElement.nativeElement);

    const mapConfig = await this.actionsService.getMapConfig();
    if (!!mapConfig) {
      this.mapService.setMapConfig(mapConfig);
    }
  }

  public ngOnDestroy() {
    for (const e of this.effects) {
      e.destroy();
    }

    this.mapService.removeMap();
  }
}
