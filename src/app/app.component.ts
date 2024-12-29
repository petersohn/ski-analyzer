import { Component, Signal } from "@angular/core";
import { MatProgressBarModule } from "@angular/material/progress-bar";
import { MainMenuComponent } from "@/components/main-menu.component";
import { MapComponent } from "@/components/map.component";
import { SkiAreaSelectorComponent } from "@/components/ski-area-selector.component";
import { SelectionInfoComponent } from "@/components/selection-info.component";
import { ActionsService } from "@/services/actions.service";
import { SkiAreaChooserService } from "@/services/ski-area-chooser.service";
import { CommonModule } from "@angular/common";
import { MatIconRegistry } from "@angular/material/icon";
import { DomSanitizer, SafeResourceUrl } from "@angular/platform-browser";

@Component({
  selector: "app-root",
  standalone: true,
  imports: [
    MainMenuComponent,
    MapComponent,
    SelectionInfoComponent,
    SkiAreaSelectorComponent,
    MatProgressBarModule,
    CommonModule,
  ],
  templateUrl: "./app.component.html",
  styleUrls: ["./app.component.css"],
})
export class AppComponent {
  public loading: Signal<boolean>;
  public hasSelectableSkiArea: Signal<boolean>;

  constructor(
    private readonly actionsService: ActionsService,
    private readonly skiAreaChooserService: SkiAreaChooserService,
    private readonly matIconRegistry: MatIconRegistry,
    private readonly domSanitizer: DomSanitizer,
  ) {
    this.loading = this.actionsService.loading;
    this.hasSelectableSkiArea = this.skiAreaChooserService.hasChoosableSkiArea;
    this.initIcons();
  }

  private sanitize(s: string): SafeResourceUrl {
    return this.domSanitizer.bypassSecurityTrustResourceUrl(s);
  }

  private initIcons() {
    this.matIconRegistry
      .addSvgIcon("piste", this.sanitize("/assets/piste.svg"))
      .addSvgIcon("cablecar", this.sanitize("/assets/lift/cablecar.svg"))
      .addSvgIcon("chairlift", this.sanitize("/assets/lift/chairlift.svg"))
      .addSvgIcon("draglift", this.sanitize("/assets/lift/draglift.svg"))
      .addSvgIcon("gondola", this.sanitize("/assets/lift/gondola.svg"))
      .addSvgIcon("zipline", this.sanitize("/assets/lift/zipline.svg"));
  }
}
