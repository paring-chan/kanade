import { AlertDialog, Button, Combobox, Dialog, Field } from "@base-ui/react";
import { useForm } from "@tanstack/react-form";
import {
  useQuery,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { type } from "arktype";
import { toast } from "sonner";
import { button, combobox, dialog, formField, input } from "../components";
import { teamListQueryOptions } from "../queries/team";

import LuX from "~icons/lucide/x";
import LuChevronDown from "~icons/lucide/chevron-down";
import LuCheck from "~icons/lucide/check";
import LuEdit from "~icons/lucide/pencil";
import LuTrash from "~icons/lucide/trash";

import { useMemo, type SubmitEvent } from "react";
import { api } from "../utils/api";
import { agentsQueryOptions } from "../queries/agent";

import type { components } from "../utils/api/types";

export const Component = () => {
  return (
    <div className="px-4">
      <div className="container mx-auto mt-12">
        <div className="flex items-center">
          <h1 className="text-3xl grow w-0">에이전트 목록</h1>
          <CreateAgentDialog />
        </div>

        <AgentList />
      </div>
      <DeleteAgentDialog />
    </div>
  );
};

export const AgentList = () => {
  const { data: agents } = useSuspenseQuery(agentsQueryOptions());

  return (
    <div className="mt-4">
      {agents.map((x) => (
        <AgentItem key={x.id} agent={x} />
      ))}
    </div>
  );
};

const AgentItem = ({
  agent,
}: {
  agent: components["schemas"]["AgentResponse"];
}) => {
  return (
    <div className="border -mt-px border-gray-300 px-4 py-2 flex items-center gap-4">
      <div className="truncate">{agent.name}</div>

      <div className="grow" />
      <div className="text-sm opacity-40">
        {agent.status === "offline"
          ? "오프라인"
          : agent.status === "busy"
            ? "작업 처리 중"
            : agent.status === "idle"
              ? "온라인"
              : "알 수 없음"}
      </div>
      <div className="flex items-center gap-2">
        <button className="opacity-40 hover:opacity-100 focus-visible:opacity-100 pointer cursor-pointer transition-opacity">
          <LuEdit className="size-4" />
        </button>
        <button
          className="opacity-40 hover:opacity-100 text-red-400 focus-visible:opacity-100 pointer cursor-pointer transition-opacity"
          onClick={() => {
            deleteHandle.openWithPayload({ agent });
          }}
        >
          <LuTrash className="size-4" />
        </button>
      </div>
    </div>
  );
};

const createAgentSchema = type({
  name: "string > 0 & string <= 20",
  teamId: "string == 36",
});

const deleteHandle = AlertDialog.createHandle<{
  agent: components["schemas"]["AgentResponse"];
}>();

const DeleteAgentDialog = () => {
  const qc = useQueryClient();

  const onSubmit = async (e: SubmitEvent<HTMLFormElement>) => {
    try {
      const value = new FormData(e.currentTarget);
      const id = value.get("id") as string;
      if (!id) throw new Error("id not defined");

      await api.DELETE("/api/v1/agents/{agent_id}", {
        params: { path: { agent_id: id } },
      });

      toast.success("에이전트가 삭제되었습니다");
      qc.invalidateQueries(agentsQueryOptions());

      deleteHandle.close();
    } catch (e: any) {
      if (e.message) toast.error(e.message);
    }
  };

  return (
    <AlertDialog.Root handle={deleteHandle}>
      {({ payload }) => (
        <AlertDialog.Portal>
          <AlertDialog.Backdrop className={dialog.backdrop()} />
          <AlertDialog.Popup
            className={dialog.popup()}
            render={
              <form
                onSubmit={(e) => {
                  e.preventDefault();
                  onSubmit(e);
                }}
              />
            }
          >
            <input type="hidden" name="id" value={payload?.agent.id} />
            <div className={dialog.titleArea()}>
              <AlertDialog.Title className={dialog.title()}>
                에이전트 삭제
              </AlertDialog.Title>
            </div>
            <div className={dialog.actionsArea()}>
              <Dialog.Close className={button({ style: "outlined" })}>
                취소
              </Dialog.Close>
              <button className={button({ style: "error" })} type="submit">
                삭제
              </button>
            </div>
          </AlertDialog.Popup>
        </AlertDialog.Portal>
      )}
    </AlertDialog.Root>
  );
};

const CreateAgentDialog = () => {
  const qc = useQueryClient();
  const dialogHandle = useMemo(() => Dialog.createHandle(), []);
  const createdHandle = useMemo(
    () => AlertDialog.createHandle<{ name: string; token: string }>(),
    [],
  );

  const form = useForm({
    defaultValues: { name: "", teamId: "" } as type.infer<
      typeof createAgentSchema
    >,
    onSubmit: async ({ value }) => {
      try {
        const res = await api
          .POST("/api/v1/agents", {
            body: {
              name: value.name,
              teamId: value.teamId,
            },
          })
          .then((x) => x.data!);

        createdHandle.openWithPayload({ name: res.name, token: res.token });
        qc.invalidateQueries(agentsQueryOptions());

        dialogHandle.close();
      } catch (e: any) {
        if (e.message) toast.error(e.message);
      }
    },

    validators: {
      onChange: createAgentSchema,
    },
  });

  return (
    <>
      <Dialog.Root handle={dialogHandle}>
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
                에이전트 생성
              </Dialog.Title>
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
                      에이전트 이름
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
                name="teamId"
                children={(field) => (
                  <Field.Root className={formField.root()}>
                    <Field.Label className={formField.label()}>
                      스코프
                    </Field.Label>
                    <ScopeSelector
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
      <AlertDialog.Root handle={createdHandle}>
        {({ payload }) => (
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
                  에이전트 생성됨
                </Dialog.Title>
                <Dialog.Description className={dialog.description()}>
                  아래 토큰으로 에이전트를 설정해주세요. 토큰은 한번만
                  표시됩니다.
                </Dialog.Description>
              </div>

              <div>
                <Field.Root className={formField.root()}>
                  <Field.Label className={formField.label()}>토큰</Field.Label>

                  <Field.Control
                    className={input()}
                    value={payload?.token}
                    readOnly
                  />
                </Field.Root>
              </div>

              <div className={dialog.actionsArea()}>
                <Dialog.Close className={button({ style: "normal" })}>
                  닫기
                </Dialog.Close>
              </div>
            </Dialog.Popup>
          </Dialog.Portal>
        )}
      </AlertDialog.Root>
    </>
  );
};

const globalItem = {
  id: "00000000-0000-0000-0000-000000000000",
  name: "전역",
  slug: "global",
};

const ScopeSelector = ({
  value,
  onChange,
}: {
  value: string;
  onChange: (value: string) => void;
}) => {
  const { data, isPending } = useQuery(teamListQueryOptions());

  const items = (() => {
    if (!data) return [];

    return [globalItem, ...data];
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
        <Combobox.Input
          placeholder="스코프 선택"
          className={combobox.input()}
        />
        <Combobox.Clear>
          <LuX />
        </Combobox.Clear>
        <Combobox.Trigger className={combobox.trigger()}>
          <LuChevronDown />
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
                    <LuCheck className="size-4" />
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
