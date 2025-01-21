import { Component, Inject, HostListener } from "@angular/core";
import { FormsModule } from "@angular/forms";
import { MAT_DIALOG_DATA, MatDialogRef } from "@angular/material/dialog";
import { MatInput } from "@angular/material/input";
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
    imports: [
        FormsModule,
        MatInput,
        MatFormFieldModule,
        MatButtonModule,
        MatDialogModule,
    ]
})
export class ConfirmationDialogComponent {
  constructor(
    @Inject(MAT_DIALOG_DATA) public data: ConfirmationDialogData,
    private readonly dialogRef: MatDialogRef<ConfirmationDialogComponent>,
  ) {}

  @HostListener("window:keyup.enter")
  public onEnter() {
    for (const option of this.data.options) {
      if (option.default) {
        this.dialogRef.close(option.value);
      }
    }
  }
}
