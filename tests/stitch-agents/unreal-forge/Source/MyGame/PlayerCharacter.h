// PlayerCharacter.h
// Test fixture for Forge agent detection (Unreal Engine)

#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Character.h"
#include "PlayerCharacter.generated.h"

UCLASS()
class MYGAME_API APlayerCharacter : public ACharacter
{
    GENERATED_BODY()

public:
    APlayerCharacter();

    // Subtle: raw pointer instead of TObjectPtr (UE5 best practice)
    UPROPERTY(EditAnywhere, BlueprintReadWrite)
    UStaticMeshComponent* WeaponMesh;
    
    // Subtle: magic numbers
    UPROPERTY(EditAnywhere)
    float MoveSpeed = 600.0f;
    
    UPROPERTY(EditAnywhere)  
    float JumpHeight = 420.0f;

protected:
    virtual void BeginPlay() override;
    virtual void Tick(float DeltaTime) override;
    
    // Subtle: non-const reference parameter
    void ProcessInput(FVector& InputVector);
    
    // Subtle: returning raw pointer
    UActorComponent* FindComponentByTag(FName Tag);

private:
    // Subtle: mutable state without synchronization
    int32 FrameCounter;
    
    // Subtle: C-style array instead of TArray
    float RecentSpeeds[10];
    int SpeedIndex;
    
    void UpdateSpeedHistory(float Speed);
};
