import { Button, Combobox, Dialog, Field } from "@base-ui/react";
import { button } from "../button";
import { dialog } from "../dialog";
import { useForm } from "@tanstack/react-form";
import { combobox } from "../combobox";

import LucideX from "~icons/lucide/x";
import LucideChevronDown from "~icons/lucide/chevron-down";
import LucideCheck from "~icons/lucide/check";

import { useQuery } from "@tanstack/react-query";
import { teamListQueryOptions } from "../../queries/team";
import { type } from "arktype";
import { formField, input } from "../form";
import { userForgesQueryOptions } from "../../queries/user";

const createProjectSchema = type({
  name: "string > 0",
  slug: "/^[a-zA-Z0-9-_]{1,20}$/",
  teamId: "string == 36",
  forgeId: "string==36",
  forgeRepoId: "string > 0",
});

export const CreateProjectDialog = ({
  defaultTeamId,
}: {
  defaultTeamId?: string;
}) => {
  const form = useForm({
    defaultValues: {
      teamId: defaultTeamId,
      slug: "",
      name: "",
      forgeRepoId: "",
      forgeId: "",
    } as type.infer<typeof createProjectSchema>,
    validators: {
      onChange: createProjectSchema,
    },
  });

  return (
    <Dialog.Root
      onOpenChange={(open) => {
        if (open) {
          form.reset();
        }
      }}
    >
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
            <Dialog.Title className={dialog.title()}>
              프로젝트 생성
            </Dialog.Title>
            <Dialog.Description className={dialog.description()}>
              선택한 포지의 저장소에 대해 CI를 활성화합니다. 저장소 소유자
              권한이 있는 경우에만 목록에 표시됩니다.
            </Dialog.Description>
          </div>

          <form.Subscribe
            selector={(state) => state.values}
            children={(values) => <pre>{JSON.stringify(values, null, 2)}</pre>}
          />

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
                    프로젝트 이름
                  </Field.Label>
                  <Field.Control
                    className={input()}
                    value={field.state.value}
                    onValueChange={field.handleChange}
                    onBlur={field.handleBlur}
                    placeholder="Project Mizuki"
                  />
                  <div className={formField.helperArea()}>
                    <Field.Description className={formField.description()}>
                      최대 20자
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
                    프로젝트 슬러그
                  </Field.Label>
                  <Field.Control
                    className={input()}
                    value={field.state.value}
                    onValueChange={field.handleChange}
                    onBlur={field.handleBlur}
                    placeholder="project-mizuki"
                  />
                  <div className={formField.helperArea()}>
                    <Field.Description className={formField.description()}>
                      최대 20자, 알파벳, 숫자, -, _만 사용 가능
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
              name="teamId"
              children={(field) => (
                <Field.Root className={formField.root()}>
                  <Field.Label className={formField.label()}>팀</Field.Label>
                  <TeamSelector
                    value={field.state.value}
                    onChange={(value) => {
                      field.setValue(value);
                    }}
                  />

                  <div className={formField.helperArea()}>
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
              name="forgeId"
              children={(forgeField) => (
                <>
                  <Field.Root className={formField.root()}>
                    <Field.Label className={formField.label()}>
                      포지
                    </Field.Label>
                    <ForgeSelector
                      value={forgeField.state.value}
                      onChange={(value) => {
                        forgeField.setValue(value);
                      }}
                    />

                    <div className={formField.helperArea()}>
                      <Field.Error
                        className={formField.error()}
                        match={!forgeField.state.meta.isValid}
                      >
                        {forgeField.state.meta.errors.join(",")}
                      </Field.Error>
                    </div>
                  </Field.Root>
                  <form.Field
                    name="forgeRepoId"
                    children={(repoField) => (
                      <Field.Root className={formField.root()}>
                        <Field.Label className={formField.label()}>
                          저장소
                        </Field.Label>
                        <RepositorySelector
                          forgeId={forgeField.state.value}
                          value={repoField.state.value}
                          onChange={(value) => {
                            repoField.setValue(value);
                          }}
                        />

                        <div className={formField.helperArea()}>
                          <Field.Error
                            className={formField.error()}
                            match={!repoField.state.meta.isValid}
                          >
                            {repoField.state.meta.errors.join(",")}
                          </Field.Error>
                        </div>
                      </Field.Root>
                    )}
                  />
                </>
              )}
            />
          </div>

          <div className={dialog.actionsArea()}>
            <Dialog.Close className={button({ style: "outlined" })}>
              취소
            </Dialog.Close>
            <Button type="submit" className={button({ style: "normal" })}>
              생성
            </Button>
          </div>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog.Root>
  );
};

const TeamSelector = ({
  value,
  onChange,
}: {
  value: string;
  onChange: (value: string) => void;
}) => {
  const { data, isPending } = useQuery(teamListQueryOptions());

  const items = (() => {
    if (!data) return [];

    return data;
  })();

  const current = items.find((x) => x.id === value) ?? null;

  type Item = (typeof items)[number];

  return (
    <Combobox.Root
      value={current}
      onValueChange={(value) => onChange(value?.id || "")}
      items={items}
      disabled={isPending}
      itemToStringLabel={(x: Item) => x.name}
      filter={(item: Item, query) => {
        if (item.slug.includes(query)) return true;
        if (item.name.includes(query)) return true;
        return false;
      }}
    >
      <Combobox.InputGroup className={combobox.inputGroup()}>
        <Combobox.Input placeholder="팀 선택" className={combobox.input()} />
        <Combobox.Clear>
          <LucideX />
        </Combobox.Clear>
        <Combobox.Trigger className={combobox.trigger()}>
          <LucideChevronDown />
        </Combobox.Trigger>
      </Combobox.InputGroup>

      <Combobox.Portal>
        <Combobox.Positioner className={combobox.positioner()} sideOffset={4}>
          <Combobox.Popup className={combobox.popup()}>
            <Combobox.Empty>
              <div className={combobox.empty()}>결과가 없습니다</div>
            </Combobox.Empty>
            <Combobox.List>
              {(item: Item) => (
                <Combobox.Item
                  key={item.id}
                  value={item}
                  className={combobox.item()}
                >
                  <Combobox.ItemIndicator className={combobox.itemIndicator()}>
                    <LucideCheck className="size-4" />
                  </Combobox.ItemIndicator>
                  <div className={combobox.itemContent()}>
                    <div>{item.name}</div>
                    <div className="text-xs opacity-60">{item.slug}</div>
                  </div>
                </Combobox.Item>
              )}
            </Combobox.List>
          </Combobox.Popup>
        </Combobox.Positioner>
      </Combobox.Portal>
    </Combobox.Root>
  );
};

const ForgeSelector = ({
  value,
  onChange,
}: {
  value: string;
  onChange: (value: string) => void;
}) => {
  const { data, isPending } = useQuery(userForgesQueryOptions());

  const items = (() => {
    if (!data) return [];

    return data;
  })();

  const current = items.find((x) => x.forge.id === value) ?? null;

  type Item = (typeof items)[number];

  return (
    <Combobox.Root
      value={current}
      onValueChange={(value) => onChange(value?.forge.id || "")}
      items={items}
      disabled={isPending}
      itemToStringLabel={(x: Item) => x.forge.name}
      filter={(item: Item, query) => {
        if (item.forge.name.includes(query)) return true;
        return false;
      }}
    >
      <Combobox.InputGroup className={combobox.inputGroup()}>
        <Combobox.Input placeholder="포지 선택" className={combobox.input()} />
        <Combobox.Clear>
          <LucideX />
        </Combobox.Clear>
        <Combobox.Trigger className={combobox.trigger()}>
          <LucideChevronDown />
        </Combobox.Trigger>
      </Combobox.InputGroup>

      <Combobox.Portal>
        <Combobox.Positioner className={combobox.positioner()} sideOffset={4}>
          <Combobox.Popup className={combobox.popup()}>
            <Combobox.Empty>
              <div className={combobox.empty()}>결과가 없습니다</div>
            </Combobox.Empty>
            <Combobox.List>
              {(item: Item) => (
                <Combobox.Item
                  key={item.id}
                  value={item}
                  className={combobox.item()}
                >
                  <Combobox.ItemIndicator className={combobox.itemIndicator()}>
                    <LucideCheck className="size-4" />
                  </Combobox.ItemIndicator>
                  <div className={combobox.itemContent()}>
                    <div>{item.forge.name}</div>
                  </div>
                </Combobox.Item>
              )}
            </Combobox.List>
          </Combobox.Popup>
        </Combobox.Positioner>
      </Combobox.Portal>
    </Combobox.Root>
  );
};

const RepositorySelector = ({
  forgeId,
}: {
  value: string;
  onChange: (value: string) => void;
  forgeId: string;
}) => {
  return (
    <Combobox.Root disabled={!forgeId}>
      <Combobox.InputGroup className={combobox.inputGroup()}>
        <Combobox.Input
          placeholder="저장소 선택"
          className={combobox.input()}
        />
        <Combobox.Clear>
          <LucideX />
        </Combobox.Clear>
        <Combobox.Trigger className={combobox.trigger()}>
          <LucideChevronDown />
        </Combobox.Trigger>
      </Combobox.InputGroup>

      <Combobox.Portal>
        <Combobox.Positioner className={combobox.positioner()} sideOffset={4}>
          <Combobox.Popup className={combobox.popup()}>
            <Combobox.Empty>
              <div className={combobox.empty()}>결과가 없습니다</div>
            </Combobox.Empty>
          </Combobox.Popup>
        </Combobox.Positioner>
      </Combobox.Portal>
    </Combobox.Root>
  );
};
