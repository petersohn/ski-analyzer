import {
  Component,
  AfterViewInit,
  OnDestroy,
  ElementRef,
  ViewChild,
} from "@angular/core";
import { MapService } from "@/services/map.service";
import { invoke } from "@tauri-apps/api/core";
import { RawSkiArea } from "@/types/skiArea";
import { RawTrack } from "@/types/track";

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

  constructor(private readonly mapService: MapService) {}

  public async ngAfterViewInit() {
    this.mapService.createMap(this.mapElement.nativeElement);
    const ski_area = await invoke("get_active_ski_area", {});
    if (!!ski_area) {
      this.mapService.loadSkiArea(ski_area as RawSkiArea);
    }

    const route = await invoke("get_active_route", {});
    if (!!route) {
      this.mapService.loadTrack(JSON.parse(route as string) as RawTrack);
    }
  }

  public ngOnDestroy() {
    this.mapService.removeMap();
  }
}
