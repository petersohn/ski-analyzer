import {
  Component,
  ChangeDetectorRef,
  ChangeDetectionStrategy,
  ViewChild,
} from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { MatProgressBarModule } from "@angular/material/progress-bar";
import { MainMenuComponent } from "./main-menu.component";
import { MapComponent } from "./map.component";
import { SkiArea } from "./types/skiArea";

@Component({
  selector: "app-root",
  standalone: true,
  imports: [MainMenuComponent, MapComponent, MatProgressBarModule],
  templateUrl: "./app.component.html",
  styleUrls: ["./app.component.css"],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class AppComponent {
  @ViewChild(MapComponent)
  public map!: MapComponent;

  public loading = false;

  constructor(private changeDetector: ChangeDetectorRef) {}

  public async loadSkiArea(path: string) {
    const data = JSON.parse(await invoke("load_file", { path }));
    this.map.loadSkiArea(data as SkiArea);
  }

  public async findSkiArea(name: string) {
    const data = await this.doJob(invoke("find_ski_area", { name }));
    this.map.loadSkiArea(data as SkiArea);
  }

  private async doJob<T>(job: Promise<T>): Promise<T> {
    this.setLoading(true);
    try {
      return await job;
    } finally {
      this.setLoading(false);
    }
  }

  private setLoading(value: boolean) {
    if (value != this.loading) {
      this.loading = value;
      this.changeDetector.detectChanges();
    }
  }
}
