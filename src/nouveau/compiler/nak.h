/*
 * Copyright © 2022 Collabora, Ltd.
 * SPDX-License-Identifier: MIT
 */

#ifndef NAK_H
#define NAK_H

#include <assert.h>
#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

struct nir_shader_compiler_options;
typedef struct nir_shader nir_shader;

struct nak_compiler {
};

void nak_optimize_nir(nir_shader *nir, const struct nak_compiler *nak);
void nak_preprocess_nir(nir_shader *nir, const struct nak_compiler *nak);
void nak_postprocess_nir(nir_shader *nir, const struct nak_compiler *nak);

struct nak_shader_info {
   /** Number of GPRs used */
   uint8_t num_gprs;

   /** Size of thread-local storage */
   uint32_t tls_size;

   struct {
      /* Local workgroup size */
      uint16_t local_size[3];

      /* Shared memory size */
      uint16_t smem_size;
   } cs;

   /** Shader header for 3D stages */
   uint32_t hdr[32];
};

struct nak_shader_bin {
   struct nak_shader_info info;
   uint32_t code_size;
   const void *code;
};

void nak_shader_bin_destroy(struct nak_shader_bin *bin);

struct nak_shader_bin *
nak_compile_shader(nir_shader *nir, const struct nak_compiler *nak);

#ifdef __cplusplus
}
#endif

#endif /* NAK_H */