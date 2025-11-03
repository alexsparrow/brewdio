import { type UIMessage } from "@ai-sdk/react";
import {
  convertToModelMessages,
  stepCountIs,
  streamText,
  type ChatRequestOptions,
  type ChatTransport,
  type LanguageModel,
  type ToolSet,
  type UIMessageChunk,
} from "ai";

export class CustomChatTransport implements ChatTransport<UIMessage> {
  private model: LanguageModel;
  private tools: ToolSet;

  constructor(model: LanguageModel, tools: ToolSet) {
    this.model = model;
    this.tools = tools;
  }

  updateModel(model: LanguageModel) {
    this.model = model;
  }
  
  updateTools(tools: ToolSet) {
    this.tools = tools;
  }

  async sendMessages(
    options: {
      chatId: string;
      messages: UIMessage[];
      abortSignal: AbortSignal | undefined;
    } & {
      trigger: "submit-message" | "regenerate-message";
      messageId: string | undefined;
    } & ChatRequestOptions
  ): Promise<ReadableStream<UIMessageChunk>> {
    console.log('[CustomChatTransport] Starting with maxToolRoundtrips: 5');

    const result = streamText({
      model: this.model,
      system: `You are a helpful brewing assistant. When using tools:
- Be efficient - complete tasks in as few steps as possible
- NEVER call the same tool multiple times in a row
- After calling getCurrentRecipe ONCE, you have the recipe data - use it directly, do NOT call it again
- After getting data from any tool, use it immediately - don't retrieve it again
- When you have enough information to answer, STOP calling tools and respond to the user`,
      messages: convertToModelMessages(options.messages),
      abortSignal: options.abortSignal,
      toolChoice: "auto",
      tools: this.tools,
      stopWhen: stepCountIs(5),
      onStepFinish: (step) => {
        console.log(`[CustomChatTransport] Step finished:`, {
          toolCalls: step.toolCalls?.length || 0,
          toolResults: step.toolResults?.length || 0,
        });
      },
    });

    return result.toUIMessageStream({
      onError: (error) => {
        // Note: By default, the AI SDK will return "An error occurred",
        // which is intentionally vague in case the error contains sensitive information like API keys.
        // If you want to provide more detailed error messages, keep the code below. Otherwise, remove this whole onError callback.
        if (error == null) {
          return "Unknown error";
        }
        if (typeof error === "string") {
          return error;
        }
        if (error instanceof Error) {
          return error.message;
        }
        return JSON.stringify(error);
      },
    });
  }

  // No-op since we don't have a backend to restore streams from
  async reconnectToStream(
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    _options: {
      chatId: string;
    } & ChatRequestOptions
  ): Promise<ReadableStream<UIMessageChunk> | null> {
    // This function normally handles reconnecting to a stream on the backend, e.g. /api/chat
    // Since this project has no backend, we can't reconnect to a stream, so this is intentionally no-op.
    return null;
  }
}
