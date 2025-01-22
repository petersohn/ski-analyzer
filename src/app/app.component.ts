import { Component, OnInit, OnDestroy, Signal } from "@angular/core";
import { CommonModule } from "@angular/common";
import { DomSanitizer, SafeResourceUrl } from "@angular/platform-browser";
import { MatProgressBarModule } from "@angular/material/progress-bar";
import { MainMenuComponent } from "@/components/main-menu.component";
import { MapComponent } from "@/components/map.component";
import { SkiAreaSelectorComponent } from "@/components/ski-area-selector.component";
import { SelectionInfoComponent } from "@/components/selection-info.component";
import { ActionsService } from "@/services/actions.service";
import { SkiAreaChooserService } from "@/services/ski-area-chooser.service";
import { EventsService } from "@/services/events.service";
import { MatIconRegistry } from "@angular/material/icon";

@Component({
  selector: "app-root",
  imports: [
    MainMenuComponent,
    MapComponent,
    SelectionInfoComponent,
    SkiAreaSelectorComponent,
    CommonModule,
    MatProgressBarModule,
  ],
  templateUrl: "./app.component.html",
  styleUrls: ["./app.component.scss"],
})
export class AppComponent implements OnInit, OnDestroy {
  public loading: Signal<boolean>;
  public hasSelectableSkiArea: Signal<boolean>;

  constructor(
    private readonly actionsService: ActionsService,
    private readonly eventsService: EventsService,
    private readonly skiAreaChooserService: SkiAreaChooserService,
    private readonly matIconRegistry: MatIconRegistry,
    private readonly domSanitizer: DomSanitizer,
  ) {
    this.loading = this.actionsService.loading;
    this.hasSelectableSkiArea = this.skiAreaChooserService.hasChoosableSkiArea;
    this.initIcons();
  }

  public async ngOnInit() {
    await this.eventsService.initEvents();
  }

  public ngOnDestroy() {
    this.eventsService.deinitEvents();
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
