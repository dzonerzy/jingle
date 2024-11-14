
#include "context.h"

#include "jingle_assembly_emitter.h"
#include "jingle_pcode_emitter.h"
#include "jingle_sleigh/src/ffi/instruction.rs.h"
#include "sleigh/globalcontext.hh"
#include "sleigh/loadimage.hh"
#include "sleigh/xml.hh"
#include "varnode_translation.h"
#include "rust_load_image.h"
#include <memory>
#include <utility>

ContextFFI::ContextFFI(rust::Slice<const rust::u8> slabytes)
    : sleigh(new DummyLoadImage(), &c_db)
{
  ghidra::AttributeId::initialize();
  ghidra::ElementId::initialize();
  this->slaBytes = std::vector<uint8_t>(slabytes.data(), slabytes.data() + slabytes.size());
  sleigh.initialize(this->slaBytes);
}

void ContextFFI::set_initial_context(rust::Str name, uint32_t val)
{
  sleigh.setContextDefault(name.operator std::string(), val);
}

std::shared_ptr<AddrSpaceHandle>
ContextFFI::getSpaceByIndex(ghidra::int4 idx) const
{
  return std::make_shared<AddrSpaceHandle>(sleigh.getSpace(idx));
}

ghidra::int4 ContextFFI::getNumSpaces() const { return sleigh.numSpaces(); }

VarnodeInfoFFI ContextFFI::getRegister(rust::Str name) const
{
  ghidra::VarnodeData vn = sleigh.getRegister(name.operator std::string());
  VarnodeInfoFFI info;
  info.space = std::make_unique<AddrSpaceHandle>(vn.space);
  info.size = vn.size;
  info.offset = vn.offset;
  return info;
};

rust::Str ContextFFI::getRegisterName(VarnodeInfoFFI vn) const
{
  std::string name =
      sleigh.getRegisterName(vn.space->getRaw(), vn.offset, vn.size);
  return {name};
}

rust::Vec<RegisterInfoFFI> ContextFFI::getRegisters() const
{
  std::map<ghidra::VarnodeData, std::string> reglist;
  rust::Vec<RegisterInfoFFI> v;
  sleigh.getAllRegisters(reglist);
  v.reserve(reglist.size());
  for (auto const &vn : reglist)
  {
    v.emplace_back(collectRegInfo(vn));
  }
  return v;
}

void ContextFFI::setImage(ImageFFI const &img)
{
  sleigh.reset(new RustLoadImage(img), &c_db);
  sleigh.initialize(this->slaBytes);
}

InstructionFFI ContextFFI::get_one_instruction(uint64_t offset) const
{
  JinglePcodeEmitter pcode;
  JingleAssemblyEmitter assembly;
  ghidra::Address a = ghidra::Address(sleigh.getDefaultCodeSpace(), offset);
  sleigh.printAssembly(assembly, a);
  sleigh.oneInstruction(pcode, a);
  size_t length = sleigh.instructionLength(a);
  InstructionFFI i;
  Disassembly d;
  i.ops = std::move(pcode.ops);
  d.args = std::move(assembly.body);
  d.mnemonic = std::move(assembly.mnem);
  i.disassembly = std::move(d);
  i.address = offset;
  i.length = length;
  return i;
}

std::unique_ptr<ContextFFI> makeContext(rust::Slice<const rust::u8> slabytes)
{
  return std::make_unique<ContextFFI>(slabytes);
}
