import {
  Component,
  AfterViewInit,
  OnDestroy,
  ElementRef,
  ViewChild,
} from "@angular/core";
import { MapService } from "@/services/map.service";

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

  public ngAfterViewInit() {
    this.mapService.createMap(this.mapElement.nativeElement);
  }

  public ngOnDestroy() {
    this.mapService.removeMap();
  }
}
