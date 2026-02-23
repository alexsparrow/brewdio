import { createOpenAI } from '@ai-sdk/openai';

export function createOpenAIProvider(apiKey: string) {
  return createOpenAI({
    apiKey,
    compatibility: 'strict',
  });
}
