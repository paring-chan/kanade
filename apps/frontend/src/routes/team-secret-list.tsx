import { useQueryClient, useSuspenseQueries } from "@tanstack/react-query";
import {
  teamBySlugQueryOptions,
  teamSecretsQueryOptions,
} from "../queries/team";
import { useParams } from "react-router";
import { useForm } from "@tanstack/react-form";
import { type } from "arktype";
import { toast } from "sonner";
import { Button, Dialog, Field } from "@base-ui/react";
import { button, dialog, formField, input } from "../components";
import { useMemo } from "react";
import { api } from "../utils/api";
import type { components } from "../utils/api/types";

export const Component = () => {
  const params = useParams<"team">();
  const [{ data: team }, { data: secrets }] = useSuspenseQueries({
    queries: [
      teamBySlugQueryOptions(params.team!),
      teamSecretsQueryOptions(params.team!),
    ],
  });

  return (
    <div className="px-4">
      <div className="container mx-auto mt-12">
        <div>팀</div>
        <div className="flex items-end gap-2">
          <h1 className="text-3xl font-bold">{team.name}</h1>
          <span className="text-base opacity-60">{team.slug}</span>
        </div>

        <div className="flex items-center">
          <h1 className="text-2xl font-medium grow w-0 mt-8">시크릿 목록</h1>
          <CreateTeamSecretDialog team={team} />
        </div>

        <SecretList secrets={secrets} />
      </div>
    </div>
  );
};

const SecretList = ({
  secrets,
}: {
  secrets: components["schemas"]["SecretInfo"][];
}) => {
  return <pre>{JSON.stringify(secrets, null, 2)}</pre>;
};

const createSecretSchema = type({
  key: "string > 0 & string <= 36",
  value: "string >= 6 & string <= 300",
  scopes: "string > 1",
});

const CreateTeamSecretDialog = ({
  team,
}: {
  team: components["schemas"]["TeamResponse"];
}) => {
  const qc = useQueryClient();
  const createDialog = useMemo(() => Dialog.createHandle(), []);

  const form = useForm({
    defaultValues: { key: "", value: "" } as type.infer<
      typeof createSecretSchema
    >,
    onSubmit: async ({ value }) => {
      try {
        await api.POST("/api/v1/teams/{team_slug}/secrets", {
          params: { path: { team_slug: team.slug } },
          body: {
            key: value.key,
            value: value.value,
            // TODO
            scopes: value.scopes.split(",").map((x) => x.trim()) as any,
          },
        });

        qc.invalidateQueries(teamSecretsQueryOptions(team.slug));
        toast.success("created");
        createDialog.close();
      } catch (e: any) {
        if (e.message) toast.error(e.message);
      }
    },

    validators: {
      onChange: createSecretSchema,
    },
  });

  return (
    <Dialog.Root handle={createDialog}>
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
              name="key"
              children={(field) => (
                <Field.Root
                  className={formField.root()}
                  name={field.name}
                  invalid={!field.state.meta.isValid}
                  dirty={field.state.meta.isDirty}
                  touched={field.state.meta.isTouched}
                >
                  <Field.Label className={formField.label()}>
                    시크릿 키
                  </Field.Label>
                  <Field.Control
                    className={input()}
                    value={field.state.value}
                    onValueChange={field.handleChange}
                    onBlur={field.handleBlur}
                    placeholder="SOME_SECRET_KEY"
                  />
                  <div className={formField.helperArea()}>
                    <Field.Description className={formField.description()}>
                      최대 36자
                    </Field.Description>
                    <Field.Error
                      className={formField.error()}
                      match={!field.state.meta.isValid}
                    >
                      {field.state.meta.errors.join(",")}
                    </Field.Error>
                  </div>
                </Field.Root>
              )}
            />
            <form.Field
              name="value"
              children={(field) => (
                <Field.Root
                  className={formField.root()}
                  name={field.name}
                  invalid={!field.state.meta.isValid}
                  dirty={field.state.meta.isDirty}
                  touched={field.state.meta.isTouched}
                >
                  <Field.Label className={formField.label()}>
                    시크릿 값
                  </Field.Label>
                  <Field.Control
                    className={input()}
                    value={field.state.value}
                    onValueChange={field.handleChange}
                    onBlur={field.handleBlur}
                    placeholder="mizuki-is-kawaii"
                  />
                  <div className={formField.helperArea()}>
                    <Field.Description className={formField.description()}>
                      최대 300자
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
              name="scopes"
              children={(field) => (
                <Field.Root
                  className={formField.root()}
                  name={field.name}
                  invalid={!field.state.meta.isValid}
                  dirty={field.state.meta.isDirty}
                  touched={field.state.meta.isTouched}
                >
                  <Field.Label className={formField.label()}>
                    적용 범위
                  </Field.Label>
                  <Field.Control
                    className={input()}
                    value={field.state.value}
                    onValueChange={field.handleChange}
                    onBlur={field.handleBlur}
                    placeholder="push"
                  />
                  <div className={formField.helperArea()}>
                    <Field.Description className={formField.description()}>
                      , 구분, push tag release pull_request cron manual
                    </Field.Description>
                    <Field.Error
                      className={formField.error()}
                      match={!field.state.meta.isValid}
                    >
                      {field.state.meta.errors.join(",")}
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
