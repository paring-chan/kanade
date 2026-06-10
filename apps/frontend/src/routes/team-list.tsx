import { Button, Dialog, Field } from "@base-ui/react";
import { button } from "../components/button";
import { dialog } from "../components/dialog";
import { type } from "arktype";
import { useForm } from "@tanstack/react-form";
import { formField, input } from "../components";
import { api } from "../utils/api";
import { generatePath, useNavigate } from "react-router";
import { toast } from "sonner";

const createTeamSchema = type({
  name: "string >= 2 & string <= 20",
  slug: "/^[a-zA-Z0-9-_]{3,20}$/",
});

export const Component = () => {
  return (
    <div className="px-4">
      <div className="container mx-auto mt-12">
        <div className="flex items-center">
          <h1 className="text-3xl grow w-0">팀 목록</h1>
          <CreateTeamDialog />
        </div>
      </div>
    </div>
  );
};

const CreateTeamDialog = () => {
  const navigate = useNavigate();

  const form = useForm({
    defaultValues: { name: "", slug: "" } as type.infer<
      typeof createTeamSchema
    >,
    onSubmit: async ({ value }) => {
      try {
        const { data } = await api.POST("/api/v1/teams", {
          body: { name: value.name, slug: value.slug },
        });

        if (data) {
          navigate(generatePath("/t/:slug", { slug: data!.slug }));
        }
      } catch (e: any) {
        if (e.message) toast.error(e.message);
      }
    },

    validators: {
      onChange: createTeamSchema,
    },
  });

  return (
    <Dialog.Root>
      <Dialog.Trigger className={button({ style: "outlined" })}>
        생성
      </Dialog.Trigger>
      <Dialog.Portal>
        <Dialog.Backdrop className={dialog.backdrop()} />
        <Dialog.Popup
          className={dialog.popup()}
          render={
            <form
              onSubmit={(e) => {
                e.preventDefault();
                form.handleSubmit();
              }}
            />
          }
        >
          <div className={dialog.titleArea()}>
            <Dialog.Title className={dialog.title()}>팀 생성</Dialog.Title>
          </div>
          <div className={dialog.content()}>
            <form.Field
              name="name"
              children={(field) => (
                <Field.Root
                  className={formField.root()}
                  name={field.name}
                  invalid={!field.state.meta.isValid}
                  dirty={field.state.meta.isDirty}
                  touched={field.state.meta.isTouched}
                >
                  <Field.Label className={formField.label()}>
                    팀 이름
                  </Field.Label>
                  <Field.Control
                    className={input()}
                    value={field.state.value}
                    onValueChange={field.handleChange}
                    onBlur={field.handleBlur}
                    placeholder="Example Team"
                  />
                  <div className={formField.helperArea()}>
                    <Field.Description className={formField.description()}>
                      최소 2글자
                    </Field.Description>
                    <Field.Error
                      className={formField.error()}
                      match={!field.state.meta.isValid}
                    >
                      {field.state.meta.errors.join(",")}{" "}
                    </Field.Error>
                  </div>
                </Field.Root>
              )}
            />
            <form.Field
              name="slug"
              children={(field) => (
                <Field.Root
                  className={formField.root()}
                  name={field.name}
                  invalid={!field.state.meta.isValid}
                  dirty={field.state.meta.isDirty}
                  touched={field.state.meta.isTouched}
                >
                  <Field.Label className={formField.label()}>
                    팀 슬러그
                  </Field.Label>
                  <Field.Control
                    className={input()}
                    value={field.state.value}
                    onValueChange={field.handleChange}
                    onBlur={field.handleBlur}
                    placeholder="example-team"
                  />
                  <div className={formField.helperArea()}>
                    <Field.Description className={formField.description()}>
                      최소 2글자
                    </Field.Description>
                    <Field.Error
                      className={formField.error()}
                      match={!field.state.meta.isValid}
                    >
                      {field.state.meta.errors.join(",")}{" "}
                    </Field.Error>
                  </div>
                </Field.Root>
              )}
            />
          </div>
          <div className={dialog.actionsArea()}>
            <Dialog.Close className={button({ style: "outlined" })}>
              취소
            </Dialog.Close>
            <form.Subscribe
              selector={(state) => [state.canSubmit, state.isSubmitting]}
              children={([canSubmit]) => (
                <Button
                  type="submit"
                  className={button({ style: "normal" })}
                  disabled={!canSubmit}
                >
                  생성
                </Button>
              )}
            />
          </div>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog.Root>
  );
};
