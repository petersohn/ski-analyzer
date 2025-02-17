import {
  Component,
  ElementRef,
  Signal,
  ChangeDetectionStrategy,
  computed,
  effect,
} from "@angular/core";
import { CommonModule } from "@angular/common";
import { MatCardModule } from "@angular/material/card";
import { MatIconModule } from "@angular/material/icon";
import { MapService } from "@/services/map.service";
import { Lift, Piste } from "@/types/skiArea";
import { Activity, Waypoint } from "@/types/track";
import { NameValueComponent } from "./name-value.component";
import { Dayjs } from "dayjs";

const liftTypes: { [type: string]: string } = {
  cable_car: "Cable car",
  gondola: "Gondola",
  mixed_lift: "Mixed lfit",
  chair_lift: "Chairlift",
  drag_lift: "Draglift",
  "t-bar": "T-bar",
  "j-bar": "J-bar",
  platter: "Platter",
  rope_tow: "Rope tow",
  magic_carpet: "Magic carpet",
  zip_line: "Zipline",
};

const liftIcons: { [type: string]: string } = {
  cable_car: "cablecar",
  gondola: "gondola",
  mixed_lift: "gondola",
  chair_lift: "chairlift",
  drag_lift: "draglift",
  "t-bar": "draglift",
  "j-bar": "draglift",
  platter: "draglift",
  zip_line: "zipline",
};

const difficultyColors: { [type: string]: string } = {
  Novice: "#0a0",
  Easy: "#11f",
  Intermediate: "#f00",
  Advanced: "#000",
  Expert: "#000",
  Freeride: "#f60",
  Unknown: "#888",
};

const activityTypes: { [type: string]: string } = {
  Unknown: "unknown",
  UseLift: "Lift",
  EnterLift: "Enter Lift",
  ExitLift: "Exit Lift",
};

@Component({
  selector: "selection-info",
  imports: [CommonModule, MatCardModule, NameValueComponent, MatIconModule],
  templateUrl: "./selection-info.component.html",
  styleUrls: ["./selection-info.component.scss"],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SelectionInfoComponent {
  public selectedPiste: Signal<Piste | undefined>;
  public selectedLift: Signal<Lift | undefined>;
  public selectedActivity: Signal<Activity | undefined>;
  public selectedWaypoint: Signal<Waypoint | undefined>;
  public currentWaypointSpeed: Signal<number | null>;
  public currentWaypointInclination: Signal<number | null>;
  public currentWaypointClosestLift: Signal<{
    lift: Lift;
    distance: number;
  } | null>;

  constructor(
    private readonly mapService: MapService,
    private readonly elementRef: ElementRef<HTMLElement>,
  ) {
    this.selectedPiste = this.mapService.selectedPiste;
    this.selectedLift = this.mapService.selectedLift;
    this.selectedActivity = this.mapService.selectedActivity;
    this.selectedWaypoint = this.mapService.selectedWaypoint;
    this.currentWaypointSpeed = this.mapService.currentWaypointSpeed;
    this.currentWaypointInclination =
      this.mapService.currentWaypointInclination;
    this.currentWaypointClosestLift =
      this.mapService.currentWaypointClosestLift;
    effect(() => {
      const color =
        difficultyColors[this.selectedPiste()?.difficulty ?? "Unknown"];
      this.elementRef.nativeElement.style.setProperty(
        "--difficulty-color",
        color,
      );
    });
  }

  public pisteName = computed(() => this.getName(this.selectedPiste()));

  public lift = computed(
    () =>
      this.selectedLift() ||
      this.selectedActivity()?.useLift?.lift ||
      this.selectedActivity()?.enterLift ||
      this.selectedActivity()?.exitLift,
  );
  public liftName = computed(() => this.getName(this.lift()));
  public liftType = computed(() => liftTypes[this.lift()?.type ?? ""]);
  public liftIcon = computed(() => {
    const type = liftIcons[this.lift()?.type ?? ""];
    return type;
  });
  public stations = computed(() => {
    const stations = this.lift()?.stations ?? [];
    return stations.map((s) =>
      s.elevation === 0 ? "?" : this.meters(s.elevation),
    );
  });
  public liftLengths = computed((): string[] => {
    const lengths = this.lift()?.lengths ?? [];
    return lengths.map((l) => this.meters(l));
  });

  public activityType = computed(() => {
    const activity = this.selectedActivity();
    if (activity?.type !== "Moving") {
      return activityTypes[activity?.type ?? "Unknown"];
    }
    return this.selectedActivity()!.moving!.move_type;
  });
  public activityLength = computed(() =>
    this.meters(this.selectedActivity()?.length ?? 0),
  );
  public activityTime = computed(
    () =>
      this.getTime(this.selectedActivity()?.begin_time) +
      " - " +
      this.getTime(this.selectedActivity()?.end_time),
  );

  public waypointTime = computed(() =>
    this.getTime(this.selectedWaypoint()?.time),
  );

  public waypointAccuracy = computed(() => {
    const hdop = this.selectedWaypoint()?.hdop;
    return hdop !== undefined ? this.meters(hdop) : "";
  });

  public waypointSpeed = computed(() => {
    const speed = this.selectedWaypoint()?.speed;
    return !!speed ? this.metersPerSecond(speed) : "";
  });

  public waypointComment = computed(() => {
    const comment = this.selectedWaypoint()?.comment ?? "";
    return comment.split("\n");
  });

  public speed = computed(() => {
    const speed = this.currentWaypointSpeed();
    return !!speed ? this.metersPerSecond(speed) : "";
  });

  public elevation = computed(() => {
    const elevation = this.selectedWaypoint()?.elevation;
    return !!elevation ? this.meters(elevation) : "";
  });

  public inclination = computed(() => {
    const inclination = this.currentWaypointInclination();
    return !!inclination ? Math.round(inclination * 100) + "%" : "";
  });

  public elevationAccuracy = computed(() => {
    const vdop = this.selectedWaypoint()?.vdop;
    return !!vdop ? this.meters(vdop) : "";
  });

  public closestLift = computed(() => {
    const lift = this.currentWaypointClosestLift();
    return !!lift ? `${lift.lift.name} (${this.meters(lift.distance)})` : "";
  });

  private meters(len: number) {
    return Math.round(len) + " m";
  }

  private metersPerSecond(speed: number) {
    return speed.toFixed(1) + " m/s";
  }

  private getTime(time?: Dayjs | null): string {
    return time?.format("HH:mm:ss") ?? "???";
  }

  private getName(input?: { ref: string; name: string }) {
    if (!input) {
      return "";
    }

    if (input.ref && input.name) {
      return `[${input.ref}] ${input.name}`;
    } else {
      return input.name || input.ref || "";
    }
  }
}
