import {
  Component,
  Inject,
  HostListener,
  ElementRef,
  AfterViewInit,
  ChangeDetectionStrategy,
} from "@angular/core";
import { FormsModule } from "@angular/forms";
import { MAT_DIALOG_DATA, MatDialogRef } from "@angular/material/dialog";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MatButtonModule } from "@angular/material/button";
import { MatDialogModule } from "@angular/material/dialog";

export type ConfirmationDialogOption = {
  text: string;
  value?: string;
  default?: boolean;
};

export type ConfirmationDialogData = {
  text: string;
  options: ConfirmationDialogOption[];
};

@Component({
  selector: "confirmation-dialog",
  templateUrl: "./confirmation-dialog.component.html",
  styleUrl: "./confirmation-dialog.component.scss",
  imports: [FormsModule, MatFormFieldModule, MatButtonModule, MatDialogModule],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ConfirmationDialogComponent implements AfterViewInit {
  constructor(
    @Inject(MAT_DIALOG_DATA) public data: ConfirmationDialogData,
    private readonly dialogRef: MatDialogRef<ConfirmationDialogComponent>,
    private readonly elementRef: ElementRef<HTMLElement>,
  ) {}

  public ngAfterViewInit() {
    let width = 0;
    this.elementRef.nativeElement
      .querySelectorAll(".button button")
      .forEach((button) => {
        width = Math.max(width, (button as HTMLElement).offsetWidth);
      });
    this.elementRef.nativeElement.style.setProperty(
      "--button-width",
      width + "px",
    );
  }

  @HostListener("window:keyup.enter")
  public onEnter() {
    for (const option of this.data.options) {
      if (option.default) {
        this.dialogRef.close(option.value);
      }
    }
  }
}
