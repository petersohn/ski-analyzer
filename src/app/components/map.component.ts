import {
  Component,
  AfterViewInit,
  OnDestroy,
  ElementRef,
  ViewChild,
  effect,
} from "@angular/core";
import { MapService } from "@/services/map.service";
import { ActionsService } from "@/services/actions.service";

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

  constructor(
    private readonly mapService: MapService,
    private readonly actionsService: ActionsService,
  ) {
    effect(() => {
      const mapConfig = this.mapService.mapConfig();
      if (!!mapConfig) {
        this.actionsService.saveMapConfig(mapConfig);
      }
    });
  }

  public async ngAfterViewInit() {
    await this.mapService.createMap(this.mapElement.nativeElement);

    const mapConfig = await this.actionsService.getMapConfig();
    if (!!mapConfig) {
      this.mapService.setMapConfig(mapConfig);
    }

    const ski_area = await this.actionsService.getActiveSkiArea();
    if (!!ski_area) {
      this.mapService.loadSkiArea(ski_area, !mapConfig);
    }

    const route = await this.actionsService.getActiveRoute();
    if (!!route) {
      this.mapService.loadTrack(route);
    }
  }

  public ngOnDestroy() {
    this.mapService.removeMap();
  }
}
