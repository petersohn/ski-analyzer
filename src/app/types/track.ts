import dayjs from "dayjs";
import { Dayjs } from "dayjs";
import { Point, Rect } from "./geo";
import { Lift, Piste, SkiArea } from "./skiArea";
import {
  Activity as GpxActivity,
  ActivityType as GpxActivityType,
  AnalyzedRouteDef,
  DerivedData as GpxDerivedData,
  MoveType,
  Moving,
  UseLift,
  WaypointDef,
} from "./generated/generated";

export type RawWaypoint = WaypointDef;

export type RawUseLift = UseLift;

export type RawMoving = Moving;

export type RawActivityType = GpxActivityType;

export type RawActivity = GpxActivity;

export type RawTrack = AnalyzedRouteDef;

export type Waypoint = {
  point: Point;
  time?: Dayjs;
  elevation?: number | null;
  hdop?: number | null;
  vdop?: number | null;
  speed?: number | null;
  comment?: string | null;
};

export type Segment = Waypoint[];
export type Segments = Segment[];

export type ProcessedUseLift = {
  lift: Lift;
  begin_station: number | null;
  end_station: number | null;
  is_reverse: boolean;
};

export type ProcessedMoving = {
  move_type: string;
  piste?: Piste;
};

export type ActivityType =
  | "Unknown"
  | "UseLift"
  | "EnterLift"
  | "ExitLift"
  | "Moving";

export type Activity = {
  type: ActivityType;
  useLift?: ProcessedUseLift;
  enterLift?: Lift;
  exitLift?: Lift;
  moving?: ProcessedMoving;
  route: Segments;
  begin_time: Dayjs | null;
  end_time: Dayjs | null;
  length: number;
};

export type Track = {
  item: Activity[];
  bounding_rect: Rect;
};

export class TrackConverter {
  constructor(private readonly skiArea: SkiArea) {}

  public convertTrack(route: RawTrack): Track {
    return {
      item: route.item.map((activity) => {
        const typeInfo = activity.type;
        let activityType: ActivityType;
        let useLiftData: UseLift | undefined;
        let enterLiftData: string | undefined;
        let exitLiftData: string | undefined;
        let movingData: Moving | undefined;

        if ("Unknown" in typeInfo) {
          activityType = "Unknown";
        } else if ("UseLift" in typeInfo) {
          activityType = "UseLift";
          useLiftData = typeInfo.UseLift;
        } else if ("EnterLift" in typeInfo) {
          activityType = "EnterLift";
          enterLiftData = typeInfo.EnterLift;
        } else if ("ExitLift" in typeInfo) {
          activityType = "ExitLift";
          exitLiftData = typeInfo.ExitLift;
        } else if ("Moving" in typeInfo) {
          activityType = "Moving";
          movingData = typeInfo.Moving;
        } else {
          activityType = "Unknown";
        }

        return {
          type: activityType,
          useLift: this.convertUseLift(useLiftData),
          enterLift: this.getLift(enterLiftData),
          exitLift: this.getLift(exitLiftData),
          moving: this.convertMoving(movingData),
          route: this.convertRoute(activity.route),
          begin_time: dayjs(activity.begin_time),
          end_time: dayjs(activity.end_time),
          length: activity.length,
        };
      }),
      bounding_rect: route.bounding_rect,
    };
  }

  private getLift(liftId: string | undefined): Lift | undefined {
    if (liftId == undefined) {
      return undefined;
    }

    return this.skiArea.lifts.get(liftId);
  }

  private getPiste(pisteId: string | undefined): Piste | undefined {
    if (pisteId == undefined) {
      return undefined;
    }

    return this.skiArea.pistes.get(pisteId);
  }

  private convertUseLift(input?: RawUseLift): ProcessedUseLift | undefined {
    if (!input) {
      return;
    }

    const lift = this.getLift(input.lift_id);
    if (!lift) {
      throw new Error(`Lift not found with id: ${input.lift_id}`);
    }

    return {
      lift,
      begin_station: input.begin_station,
      end_station: input.end_station,
      is_reverse: input.is_reverse,
    };
  }

  private convertMoving(input?: RawMoving): ProcessedMoving | undefined {
    if (!input) {
      return;
    }

    const piste = this.getPiste(input.piste_id);
    if (input.piste_id !== "" && !piste) {
      console.warn(`Piste not found with id: ${input.piste_id}`);
    }

    return {
      move_type: input.move_type,
      piste,
    };
  }

  private convertRoute(route: RawWaypoint[][]): Segments {
    return route.map((s) =>
      s.map((wp) => {
        return {
          point: wp.point,
          time: wp.time !== undefined ? dayjs(wp.time) : undefined,
          elevation: wp.elevation,
          hdop: wp.hdop,
          vdop: wp.vdop,
          speed: wp.speed,
          comment: wp.comment,
        };
      }),
    );
  }

  private convertActivityType(type: RawActivityType): ActivityType {
    return (Object.keys(type)[0] ?? "Unknown") as ActivityType;
  }
}

export type DerivedData = GpxDerivedData;
