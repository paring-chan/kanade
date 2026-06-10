import type { paths } from "./types";
import createClient from "openapi-fetch";
import ky, { HTTPError, type NormalizedOptions } from "ky";
import { type } from "arktype";

const apiErrorSchema = type({
  message: "string",
});

export class ApiError extends HTTPError {
  constructor(
    message: string,
    response: Response,
    request: Request,
    options: NormalizedOptions,
  ) {
    super(response, request, options);
    this.message = message;
  }
}

export const api = createClient<paths>({
  fetch: ky.extend({
    hooks: {
      beforeRequest: [
        ({ request }) => {
          const token = localStorage.getItem("kanade.apikey");
          request.headers.set("Authorization", `Bearer ${token}`);
        },
      ],
      afterResponse: [
        async ({ response, request, options }) => {
          if (!response.ok) {
            const data = await response
              .clone()
              .json()
              .catch(() => null);
            const out = apiErrorSchema(data);
            if (!(out instanceof type.errors)) {
              throw new ApiError(out.message, response, request, options);
            }
          }
        },
      ],
    },
  }),
});
